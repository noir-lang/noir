//! SSA stands for Single Static Assignment
//! The IR presented in this module will already
//! be in SSA form and will be used to apply
//! conventional optimizations like Common Subexpression
//! elimination and constant folding.
//!
//! This module heavily borrows from Cranelift
#![allow(dead_code)]

use std::collections::BTreeSet;

use crate::{
    brillig::Brillig,
    errors::{RuntimeError, SsaReport},
};
use acvm::acir::{
    circuit::{Circuit, PublicInputs},
    native_types::Witness,
};

use noirc_errors::debug_info::DebugInfo;

use noirc_frontend::{
    hir_def::function::FunctionSignature, monomorphization::ast::Program, Visibility,
};
use tracing::{span, Level};

use self::{acir_gen::GeneratedAcir, ssa_gen::Ssa};

mod acir_gen;
pub(super) mod function_builder;
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

    let ssa_gen_span = span!(Level::TRACE, "ssa_generation");
    let ssa_gen_span_guard = ssa_gen_span.enter();
    let ssa = SsaBuilder::new(program, print_ssa_passes)?
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

    drop(ssa_gen_span_guard);

    let last_array_uses = ssa.find_last_array_uses();

    ssa.into_acir(brillig, abi_distinctness, &last_array_uses)
}

/// Compiles the [`Program`] into [`ACIR`][acvm::acir::circuit::Circuit].
///
/// The output ACIR is is backend-agnostic and so must go through a transformation pass before usage in proof generation.
#[allow(clippy::type_complexity)]
#[tracing::instrument(level = "trace", skip_all)]
pub fn create_circuit(
    program: Program,
    enable_ssa_logging: bool,
    enable_brillig_logging: bool,
) -> Result<(Circuit, DebugInfo, Vec<Witness>, Vec<Witness>, Vec<SsaReport>), RuntimeError> {
    let func_sig = program.main_function_signature.clone();
    let mut generated_acir =
        optimize_into_acir(program, enable_ssa_logging, enable_brillig_logging)?;
    let opcodes = generated_acir.take_opcodes();
    let current_witness_index = generated_acir.current_witness_index().0;
    let GeneratedAcir {
        return_witnesses,
        locations,
        input_witnesses,
        assert_messages,
        warnings,
        ..
    } = generated_acir;

    let (public_parameter_witnesses, private_parameters) =
        split_public_and_private_inputs(&func_sig, &input_witnesses);

    let public_parameters = PublicInputs(public_parameter_witnesses);
    let return_values = PublicInputs(return_witnesses.iter().copied().collect());

    let circuit = Circuit {
        current_witness_index,
        opcodes,
        private_parameters,
        public_parameters,
        return_values,
        assert_messages: assert_messages.into_iter().collect(),
    };

    // This converts each im::Vector in the BTreeMap to a Vec
    let locations = locations
        .into_iter()
        .map(|(index, locations)| (index, locations.into_iter().collect()))
        .collect();

    let mut debug_info = DebugInfo::new(locations);

    // Perform any ACIR-level optimizations
    let (optimized_circuit, transformation_map) = acvm::compiler::optimize(circuit);
    debug_info.update_acir(transformation_map);

    Ok((optimized_circuit, debug_info, input_witnesses, return_witnesses, warnings))
}

// Takes each function argument and partitions the circuit's inputs witnesses according to its visibility.
fn split_public_and_private_inputs(
    func_sig: &FunctionSignature,
    input_witnesses: &[Witness],
) -> (BTreeSet<Witness>, BTreeSet<Witness>) {
    let mut idx = 0_usize;
    if input_witnesses.is_empty() {
        return (BTreeSet::new(), BTreeSet::new());
    }

    func_sig
        .0
        .iter()
        .map(|(_, typ, visibility)| {
            let num_field_elements_needed = typ.field_count() as usize;
            let witnesses = input_witnesses[idx..idx + num_field_elements_needed].to_vec();
            idx += num_field_elements_needed;
            (visibility, witnesses)
        })
        .fold((BTreeSet::new(), BTreeSet::new()), |mut acc, (vis, witnesses)| {
            // Split witnesses into sets based on their visibility.
            if *vis == Visibility::Public {
                for witness in witnesses {
                    acc.0.insert(witness);
                }
            } else {
                for witness in witnesses {
                    acc.1.insert(witness);
                }
            }
            (acc.0, acc.1)
        })
}

// This is just a convenience object to bundle the ssa with `print_ssa_passes` for debug printing.
struct SsaBuilder {
    ssa: Ssa,
    print_ssa_passes: bool,
}

impl SsaBuilder {
    fn new(program: Program, print_ssa_passes: bool) -> Result<SsaBuilder, RuntimeError> {
        let ssa = ssa_gen::generate_ssa(program)?;
        Ok(SsaBuilder { print_ssa_passes, ssa }.print("Initial SSA:"))
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

    fn to_brillig(&self, print_brillig_trace: bool) -> Brillig {
        self.ssa.to_brillig(print_brillig_trace)
    }

    fn print(self, msg: &str) -> Self {
        if self.print_ssa_passes {
            println!("{msg}\n{}", self.ssa);
        }
        self
    }
}
