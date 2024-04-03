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
    circuit::{Circuit, ExpressionWidth, Program as AcirProgram, PublicInputs},
    native_types::Witness,
};

use noirc_errors::debug_info::{DebugFunctions, DebugInfo, DebugTypes, DebugVariables};

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
    force_brillig_output: bool,
) -> Result<Vec<GeneratedAcir>, RuntimeError> {
    let abi_distinctness = program.return_distinctness;

    let ssa_gen_span = span!(Level::TRACE, "ssa_generation");
    let ssa_gen_span_guard = ssa_gen_span.enter();
    let ssa = SsaBuilder::new(program, print_ssa_passes, force_brillig_output)?
        .run_pass(Ssa::defunctionalize, "After Defunctionalization:")
        .run_pass(Ssa::remove_paired_rc, "After Removing Paired rc_inc & rc_decs:")
        .run_pass(Ssa::inline_functions, "After Inlining:")
        // Run mem2reg with the CFG separated into blocks
        .run_pass(Ssa::mem2reg, "After Mem2Reg:")
        .try_run_pass(Ssa::evaluate_assert_constant, "After Assert Constant:")?
        .try_run_pass(Ssa::unroll_loops, "After Unrolling:")?
        .run_pass(Ssa::simplify_cfg, "After Simplifying:")
        .run_pass(Ssa::flatten_cfg, "After Flattening:")
        .run_pass(Ssa::remove_bit_shifts, "After Removing Bit Shifts:")
        // Run mem2reg once more with the flattened CFG to catch any remaining loads/stores
        .run_pass(Ssa::mem2reg, "After Mem2Reg:")
        .run_pass(Ssa::fold_constants, "After Constant Folding:")
        .run_pass(Ssa::fold_constants_using_constraints, "After Constraint Folding:")
        .run_pass(Ssa::dead_instruction_elimination, "After Dead Instruction Elimination:")
        .finish();

    let brillig = ssa.to_brillig(print_brillig_trace);

    drop(ssa_gen_span_guard);

    let last_array_uses = ssa.find_last_array_uses();

    ssa.into_acir(&brillig, abi_distinctness, &last_array_uses)
}

/// Compiles the [`Program`] into [`ACIR``][acvm::acir::circuit::Program].
///
/// The output ACIR is is backend-agnostic and so must go through a transformation pass before usage in proof generation.
#[allow(clippy::type_complexity)]
#[tracing::instrument(level = "trace", skip_all)]
pub fn create_program(
    program: Program,
    enable_ssa_logging: bool,
    enable_brillig_logging: bool,
    force_brillig_output: bool,
) -> Result<(AcirProgram, Vec<DebugInfo>, Vec<SsaReport>, Vec<Witness>, Vec<Witness>), RuntimeError>
{
    let debug_variables = program.debug_variables.clone();
    let debug_types = program.debug_types.clone();
    let debug_functions = program.debug_functions.clone();

    let func_sigs = program.function_signatures.clone();

    let recursive = program.recursive;
    let generated_acirs = optimize_into_acir(
        program,
        enable_ssa_logging,
        enable_brillig_logging,
        force_brillig_output,
    )?;
    assert_eq!(
        generated_acirs.len(),
        func_sigs.len(),
        "The generated ACIRs should match the supplied function signatures"
    );

    let mut functions = vec![];
    let mut debug_infos = vec![];
    let mut warning_infos = vec![];
    let mut main_input_witnesses = Vec::new();
    let mut main_return_witnesses = Vec::new();
    // For setting up the ABI we need separately specify main's input and return witnesses
    let mut is_main = true;
    for (acir, func_sig) in generated_acirs.into_iter().zip(func_sigs) {
        let (circuit, debug_info, warnings, input_witnesses, return_witnesses) =
            convert_generated_acir_into_circuit(
                acir,
                func_sig,
                recursive,
                // TODO: get rid of these clones
                debug_variables.clone(),
                debug_functions.clone(),
                debug_types.clone(),
            );
        functions.push(circuit);
        debug_infos.push(debug_info);
        warning_infos.extend(warnings);
        if is_main {
            // main_input_witness = circuit.re
            main_input_witnesses = input_witnesses;
            main_return_witnesses = return_witnesses;
        }
        is_main = false;
    }

    let program = AcirProgram { functions };

    Ok((program, debug_infos, warning_infos, main_input_witnesses, main_return_witnesses))
}

fn convert_generated_acir_into_circuit(
    mut generated_acir: GeneratedAcir,
    func_sig: FunctionSignature,
    recursive: bool,
    debug_variables: DebugVariables,
    debug_functions: DebugFunctions,
    debug_types: DebugTypes,
) -> (Circuit, DebugInfo, Vec<SsaReport>, Vec<Witness>, Vec<Witness>) {
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

    let locations = locations.clone();

    let (public_parameter_witnesses, private_parameters) =
        split_public_and_private_inputs(&func_sig, &input_witnesses);

    let public_parameters = PublicInputs(public_parameter_witnesses);
    let return_values = PublicInputs(return_witnesses.iter().copied().collect());

    let circuit = Circuit {
        current_witness_index,
        expression_width: ExpressionWidth::Unbounded,
        opcodes,
        private_parameters,
        public_parameters,
        return_values,
        assert_messages: assert_messages.clone().into_iter().collect(),
        recursive,
    };

    // This converts each im::Vector in the BTreeMap to a Vec
    let locations = locations
        .into_iter()
        .map(|(index, locations)| (index, locations.into_iter().collect()))
        .collect();

    let mut debug_info = DebugInfo::new(locations, debug_variables, debug_functions, debug_types);

    // Perform any ACIR-level optimizations
    let (optimized_circuit, transformation_map) = acvm::compiler::optimize(circuit);
    debug_info.update_acir(transformation_map);

    (optimized_circuit, debug_info, warnings, input_witnesses, return_witnesses)
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
    fn new(
        program: Program,
        print_ssa_passes: bool,
        force_brillig_runtime: bool,
    ) -> Result<SsaBuilder, RuntimeError> {
        let ssa = ssa_gen::generate_ssa(program, force_brillig_runtime)?;
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
