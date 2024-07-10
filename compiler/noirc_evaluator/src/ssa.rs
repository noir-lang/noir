//! SSA stands for Single Static Assignment
//! The IR presented in this module will already
//! be in SSA form and will be used to apply
//! conventional optimizations like Common Subexpression
//! elimination and constant folding.
//!
//! This module heavily borrows from Cranelift
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use crate::errors::{RuntimeError, SsaReport};
use acvm::{
    acir::{
        circuit::{
            brillig::BrilligBytecode, Circuit, ErrorSelector, ExpressionWidth,
            Program as AcirProgram, PublicInputs,
        },
        native_types::Witness,
    },
    FieldElement,
};

use noirc_errors::debug_info::{DebugFunctions, DebugInfo, DebugTypes, DebugVariables};

use noirc_frontend::ast::Visibility;
use noirc_frontend::{
    hir_def::{function::FunctionSignature, types::Type as HirType},
    monomorphization::ast::Program,
};
use tracing::{span, Level};

use self::{
    acir_gen::{Artifacts, GeneratedAcir},
    ssa_gen::Ssa,
};

mod acir_gen;
mod checks;
pub(super) mod function_builder;
pub mod ir;
mod opt;
pub mod ssa_gen;

pub(crate) struct ArtifactsAndWarnings(Artifacts, Vec<SsaReport>);

/// Optimize the given program by converting it into SSA
/// form and performing optimizations there. When finished,
/// convert the final SSA into an ACIR program and return it.
/// An ACIR program is made up of both ACIR functions
/// and Brillig functions for unconstrained execution.
pub(crate) fn optimize_into_acir(
    program: Program,
    options: &SsaEvaluatorOptions,
) -> Result<ArtifactsAndWarnings, RuntimeError> {
    let ssa_gen_span = span!(Level::TRACE, "ssa_generation");
    let ssa_gen_span_guard = ssa_gen_span.enter();
    let mut ssa = SsaBuilder::new(
        program,
        options.enable_ssa_logging,
        options.force_brillig_output,
        options.print_codegen_timings,
    )?
    .run_pass(Ssa::defunctionalize, "After Defunctionalization:")
    .run_pass(Ssa::remove_paired_rc, "After Removing Paired rc_inc & rc_decs:")
    .run_pass(Ssa::separate_runtime, "After Runtime Separation:")
    .run_pass(Ssa::resolve_is_unconstrained, "After Resolving IsUnconstrained:")
    .run_pass(Ssa::inline_functions, "After Inlining:")
    // Run mem2reg with the CFG separated into blocks
    .run_pass(Ssa::mem2reg, "After Mem2Reg:")
    .run_pass(Ssa::as_slice_optimization, "After `as_slice` optimization")
    .try_run_pass(
        Ssa::evaluate_static_assert_and_assert_constant,
        "After `static_assert` and `assert_constant`:",
    )?
    .try_run_pass(Ssa::unroll_loops_iteratively, "After Unrolling:")?
    .run_pass(Ssa::simplify_cfg, "After Simplifying:")
    .run_pass(Ssa::flatten_cfg, "After Flattening:")
    .run_pass(Ssa::remove_bit_shifts, "After Removing Bit Shifts:")
    // Run mem2reg once more with the flattened CFG to catch any remaining loads/stores
    .run_pass(Ssa::mem2reg, "After Mem2Reg:")
    // Run the inlining pass again to handle functions with `InlineType::NoPredicates`.
    // Before flattening is run, we treat functions marked with the `InlineType::NoPredicates` as an entry point.
    // This pass must come immediately following `mem2reg` as the succeeding passes
    // may create an SSA which inlining fails to handle.
    .run_pass(Ssa::inline_functions_with_no_predicates, "After Inlining:")
    .run_pass(Ssa::remove_if_else, "After Remove IfElse:")
    .run_pass(Ssa::fold_constants, "After Constant Folding:")
    .run_pass(Ssa::remove_enable_side_effects, "After EnableSideEffects removal:")
    .run_pass(Ssa::fold_constants_using_constraints, "After Constraint Folding:")
    .run_pass(Ssa::dead_instruction_elimination, "After Dead Instruction Elimination:")
    .run_pass(Ssa::array_set_optimization, "After Array Set Optimizations:")
    .finish();

    let ssa_level_warnings = ssa.check_for_underconstrained_values();
    let brillig = time("SSA to Brillig", options.print_codegen_timings, || {
        ssa.to_brillig(options.enable_brillig_logging)
    });

    drop(ssa_gen_span_guard);

    let artifacts = time("SSA to ACIR", options.print_codegen_timings, || ssa.into_acir(&brillig))?;
    Ok(ArtifactsAndWarnings(artifacts, ssa_level_warnings))
}

// Helper to time SSA passes
fn time<T>(name: &str, print_timings: bool, f: impl FnOnce() -> T) -> T {
    let start_time = chrono::Utc::now().time();
    let result = f();

    if print_timings {
        let end_time = chrono::Utc::now().time();
        println!("{name}: {} ms", (end_time - start_time).num_milliseconds());
    }

    result
}

#[derive(Default)]
pub struct SsaProgramArtifact {
    pub program: AcirProgram<FieldElement>,
    pub debug: Vec<DebugInfo>,
    pub warnings: Vec<SsaReport>,
    pub main_input_witnesses: Vec<Witness>,
    pub main_return_witnesses: Vec<Witness>,
    pub names: Vec<String>,
    pub error_types: BTreeMap<ErrorSelector, HirType>,
}

impl SsaProgramArtifact {
    fn new(
        unconstrained_functions: Vec<BrilligBytecode<FieldElement>>,
        error_types: BTreeMap<ErrorSelector, HirType>,
    ) -> Self {
        let program = AcirProgram { functions: Vec::default(), unconstrained_functions };
        Self {
            program,
            debug: Vec::default(),
            warnings: Vec::default(),
            main_input_witnesses: Vec::default(),
            main_return_witnesses: Vec::default(),
            names: Vec::default(),
            error_types,
        }
    }

    fn add_circuit(&mut self, mut circuit_artifact: SsaCircuitArtifact, is_main: bool) {
        self.program.functions.push(circuit_artifact.circuit);
        self.debug.push(circuit_artifact.debug_info);
        self.warnings.append(&mut circuit_artifact.warnings);
        if is_main {
            self.main_input_witnesses = circuit_artifact.input_witnesses;
            self.main_return_witnesses = circuit_artifact.return_witnesses;
        }
        self.names.push(circuit_artifact.name);
    }

    fn add_warnings(&mut self, mut warnings: Vec<SsaReport>) {
        self.warnings.append(&mut warnings);
    }
}

pub struct SsaEvaluatorOptions {
    /// Emit debug information for the intermediate SSA IR
    pub enable_ssa_logging: bool,

    pub enable_brillig_logging: bool,

    /// Force Brillig output (for step debugging)
    pub force_brillig_output: bool,

    /// Pretty print benchmark times of each code generation pass
    pub print_codegen_timings: bool,
}

/// Compiles the [`Program`] into [`ACIR``][acvm::acir::circuit::Program].
///
/// The output ACIR is backend-agnostic and so must go through a transformation pass before usage in proof generation.
#[tracing::instrument(level = "trace", skip_all)]
pub fn create_program(
    program: Program,
    options: &SsaEvaluatorOptions,
) -> Result<SsaProgramArtifact, RuntimeError> {
    let debug_variables = program.debug_variables.clone();
    let debug_types = program.debug_types.clone();
    let debug_functions = program.debug_functions.clone();

    let func_sigs = program.function_signatures.clone();

    let recursive = program.recursive;
    let ArtifactsAndWarnings((generated_acirs, generated_brillig, error_types), ssa_level_warnings) =
        optimize_into_acir(program, options)?;
    assert_eq!(
        generated_acirs.len(),
        func_sigs.len(),
        "The generated ACIRs should match the supplied function signatures"
    );

    let mut program_artifact = SsaProgramArtifact::new(generated_brillig, error_types);

    // Add warnings collected at the Ssa stage
    program_artifact.add_warnings(ssa_level_warnings);
    // For setting up the ABI we need separately specify main's input and return witnesses
    let mut is_main = true;
    for (acir, func_sig) in generated_acirs.into_iter().zip(func_sigs) {
        let circuit_artifact = convert_generated_acir_into_circuit(
            acir,
            func_sig,
            recursive,
            // TODO: get rid of these clones
            debug_variables.clone(),
            debug_functions.clone(),
            debug_types.clone(),
        );
        program_artifact.add_circuit(circuit_artifact, is_main);
        is_main = false;
    }

    Ok(program_artifact)
}

pub struct SsaCircuitArtifact {
    name: String,
    circuit: Circuit<FieldElement>,
    debug_info: DebugInfo,
    warnings: Vec<SsaReport>,
    input_witnesses: Vec<Witness>,
    return_witnesses: Vec<Witness>,
}

fn convert_generated_acir_into_circuit(
    mut generated_acir: GeneratedAcir<FieldElement>,
    func_sig: FunctionSignature,
    recursive: bool,
    debug_variables: DebugVariables,
    debug_functions: DebugFunctions,
    debug_types: DebugTypes,
) -> SsaCircuitArtifact {
    let opcodes = generated_acir.take_opcodes();
    let current_witness_index = generated_acir.current_witness_index().0;
    let GeneratedAcir {
        return_witnesses,
        locations,
        input_witnesses,
        assertion_payloads: assert_messages,
        warnings,
        name,
        ..
    } = generated_acir;

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
        assert_messages: assert_messages.into_iter().collect(),
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

    SsaCircuitArtifact {
        name,
        circuit: optimized_circuit,
        debug_info,
        warnings,
        input_witnesses,
        return_witnesses,
    }
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
    print_codegen_timings: bool,
}

impl SsaBuilder {
    fn new(
        program: Program,
        print_ssa_passes: bool,
        force_brillig_runtime: bool,
        print_codegen_timings: bool,
    ) -> Result<SsaBuilder, RuntimeError> {
        let ssa = ssa_gen::generate_ssa(program, force_brillig_runtime)?;
        Ok(SsaBuilder { print_ssa_passes, print_codegen_timings, ssa }.print("Initial SSA:"))
    }

    fn finish(self) -> Ssa {
        self.ssa
    }

    /// Runs the given SSA pass and prints the SSA afterward if `print_ssa_passes` is true.
    fn run_pass(mut self, pass: fn(Ssa) -> Ssa, msg: &str) -> Self {
        self.ssa = time(msg, self.print_codegen_timings, || pass(self.ssa));
        self.print(msg)
    }

    /// The same as `run_pass` but for passes that may fail
    fn try_run_pass(
        mut self,
        pass: fn(Ssa) -> Result<Ssa, RuntimeError>,
        msg: &str,
    ) -> Result<Self, RuntimeError> {
        self.ssa = time(msg, self.print_codegen_timings, || pass(self.ssa))?;
        Ok(self.print(msg))
    }

    fn print(self, msg: &str) -> Self {
        if self.print_ssa_passes {
            println!("{msg}\n{}", self.ssa);
        }
        self
    }
}
