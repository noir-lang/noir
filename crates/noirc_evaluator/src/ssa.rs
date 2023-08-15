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

use noirc_frontend::monomorphization::ast::Program;

use self::{abi_gen::gen_abi, acir_gen::GeneratedAcir, ir::function::RuntimeType, ssa_gen::Ssa};

mod abi_gen;
mod acir_gen;
pub mod ir;
mod opt;
mod ssa_builder;
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
    let mut ssa = ssa_gen::generate_ssa(program)
        .print(print_ssa_passes, "Initial SSA:")
        .defunctionalize()
        .print(print_ssa_passes, "After Defunctionalization:");

    let brillig = ssa.to_brillig(print_brillig_trace);
    if let RuntimeType::Acir = ssa.main().runtime() {
        ssa = ssa
            .inline_functions()
            .print(print_ssa_passes, "After Inlining:")
            // Run mem2reg with the CFG separated into blocks
            .mem2reg()
            .print(print_ssa_passes, "After Mem2Reg:")
            .evaluate_assert_constant()?
            .unroll_loops()?
            .print(print_ssa_passes, "After Unrolling:")
            .simplify_cfg()
            .print(print_ssa_passes, "After Simplifying:")
            // Run mem2reg before flattening to handle any promotion
            // of values that can be accessed after loop unrolling
            .mem2reg()
            .print(print_ssa_passes, "After Mem2Reg:")
            .flatten_cfg()
            .print(print_ssa_passes, "After Flattening:")
            // Run mem2reg once more with the flattened CFG to catch any remaining loads/stores
            .mem2reg()
            .print(print_ssa_passes, "After Mem2Reg:")
            .fold_constants()
            .print(print_ssa_passes, "After Constant Folding:")
            .dead_instruction_elimination()
            .print(print_ssa_passes, "After Dead Instruction Elimination:");
    }
    let last_array_uses = ssa.find_last_array_uses();
    ssa.into_acir(brillig, abi_distinctness, &last_array_uses)
}

/// Compiles the [`Program`] into [`ACIR`][acvm::acir::circuit::Circuit].
///
/// The output ACIR is is backend-agnostic and so must go through a transformation pass before usage in proof generation.
pub fn create_circuit(
    program: Program,
    enable_ssa_logging: bool,
    enable_brillig_logging: bool,
) -> Result<(Circuit, DebugInfo, Abi), RuntimeError> {
    let func_sig = program.main_function_signature.clone();
    let GeneratedAcir {
        current_witness_index,
        opcodes,
        return_witnesses,
        locations,
        input_witnesses,
        ..
    } = optimize_into_acir(program, enable_ssa_logging, enable_brillig_logging)?;

    let abi = gen_abi(func_sig, &input_witnesses, return_witnesses.clone());
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
    let debug_info = DebugInfo::new(locations);

    Ok((circuit, debug_info, abi))
}

impl Ssa {
    fn print(self, print_ssa_passes: bool, msg: &str) -> Ssa {
        if print_ssa_passes {
            println!("{msg}\n{self}");
        }
        self
    }
}
