//! SSA stands for Single Static Assignment
//! The IR presented in this module will already
//! be in SSA form and will be used to apply
//! conventional optimizations like Common Subexpression
//! elimination and constant folding.
//!
//! This module heavily borrows from Cranelift
#![allow(dead_code)]

use std::collections::BTreeSet;

use crate::errors::RuntimeError;
use acvm::acir::{
    circuit::{Circuit, PublicInputs},
    native_types::Witness,
};

use noirc_errors::debug_info::DebugInfo;

use noirc_abi::Abi;

use noirc_frontend::{hir::Context, monomorphization::ast::Program};

use self::{abi_gen::gen_abi, acir_gen::GeneratedAcir, ssa_gen::Ssa};

pub mod abi_gen;
mod acir_gen;
mod function_builder;
pub mod ir;
mod opt;
pub mod ssa_gen;

/// Optimize the given program by converting it into SSA
/// form and performing optimizations there. When finished,
/// convert the final SSA into ACIR and return it.
pub(crate) fn optimize_into_acir(
    program: Program,
    print_ssa_passes: bool,
    print_brillig_trace: bool,
) -> Result<GeneratedAcir, RuntimeError> {
    let abi_distinctness = program.return_distinctness;
    let ssa = SsaBuilder::new(program, print_ssa_passes)
        .run_pass(Ssa::defunctionalize, "After Defunctionalization:")
        .run_pass(Ssa::inline_functions, "After Inlining:")
        // Run mem2reg with the CFG separated into blocks
        .run_pass(Ssa::mem2reg, "After Mem2Reg:")
        .try_run_pass(Ssa::evaluate_assert_constant, "After Assert Constant:")?
        .try_run_pass(Ssa::unroll_loops, "After Unrolling:")?
        .run_pass(Ssa::simplify_cfg, "After Simplifying:")
        // Run mem2reg before flattening to handle any promotion
        // of values that can be accessed after loop unrolling.
        // If there are slice mergers uncovered by loop unrolling
        // and this pass is missed, slice merging will fail inside of flattening.
        .run_pass(Ssa::mem2reg, "After Mem2Reg:")
        .run_pass(Ssa::flatten_cfg, "After Flattening:")
        // Run mem2reg once more with the flattened CFG to catch any remaining loads/stores
        .run_pass(Ssa::mem2reg, "After Mem2Reg:")
        .run_pass(Ssa::fold_constants, "After Constant Folding:")
        .run_pass(Ssa::dead_instruction_elimination, "After Dead Instruction Elimination:")
        .finish();

    let brillig = ssa.to_brillig(print_brillig_trace);
    let last_array_uses = ssa.find_last_array_uses();
    ssa.into_acir(brillig, abi_distinctness, &last_array_uses)
}

/// Compiles the [`Program`] into [`ACIR`][acvm::acir::circuit::Circuit].
///
/// The output ACIR is is backend-agnostic and so must go through a transformation pass before usage in proof generation.
pub fn create_circuit(
    context: &Context,
    program: Program,
    enable_ssa_logging: bool,
    enable_brillig_logging: bool,
) -> Result<(Circuit, DebugInfo, Abi), RuntimeError> {
    let func_sig = program.main_function_signature.clone();
    let mut generated_acir =
        optimize_into_acir(program, enable_ssa_logging, enable_brillig_logging)?;
    let opcodes = generated_acir.take_opcodes();
    let GeneratedAcir {
        current_witness_index, return_witnesses, locations, input_witnesses, ..
    } = generated_acir;

    // TODO(Maddiaa): gen abi needs more from the program to determine the abi
    let abi = gen_abi(context, func_sig, &input_witnesses, return_witnesses.clone());
    let public_abi = abi.clone().public_abi();

    let public_parameters =
        PublicInputs(public_abi.param_witnesses.values().flatten().copied().collect());

    let all_parameters: BTreeSet<Witness> =
        abi.param_witnesses.values().flatten().copied().collect();
    let private_parameters = all_parameters.difference(&public_parameters.0).copied().collect();

    let return_values = PublicInputs(return_witnesses.into_iter().collect());

    let circuit = Circuit {
        current_witness_index,
        opcodes,
        private_parameters,
        public_parameters,
        return_values,
    };

    // This converts each im::Vector in the BTreeMap to a Vec
    let locations = locations
        .into_iter()
        .map(|(index, locations)| (index, locations.into_iter().collect()))
        .collect();

    let debug_info = DebugInfo::new(locations);

    Ok((circuit, debug_info, abi))
}

// This is just a convenience object to bundle the ssa with `print_ssa_passes` for debug printing.
struct SsaBuilder {
    ssa: Ssa,
    print_ssa_passes: bool,
}

impl SsaBuilder {
    fn new(program: Program, print_ssa_passes: bool) -> SsaBuilder {
        SsaBuilder { print_ssa_passes, ssa: ssa_gen::generate_ssa(program) }.print("Initial SSA:")
    }

    fn finish(self) -> Ssa {
        self.ssa
    }

    /// Runs the given SSA pass and prints the SSA afterward if `print_ssa_passes` is true.
    fn run_pass(mut self, pass: fn(Ssa) -> Ssa, msg: &str) -> Self {
        self.ssa = pass(self.ssa);
        self.print(msg)
    }

    /// The same as `run_pass` but for passes that may fail
    fn try_run_pass(
        mut self,
        pass: fn(Ssa) -> Result<Ssa, RuntimeError>,
        msg: &str,
    ) -> Result<Self, RuntimeError> {
        self.ssa = pass(self.ssa)?;
        Ok(self.print(msg))
    }

    fn print(self, msg: &str) -> Self {
        if self.print_ssa_passes {
            println!("{msg}\n{}", self.ssa);
        }
        self
    }
}
