//! SSA stands for Single Static Assignment
//! The IR presented in this module will already
//! be in SSA form and will be used to apply
//! conventional optimizations like Common Subexpression
//! elimination and constant folding.
//!
//! This module heavily borrows from Cranelift

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    acir::ssa::Artifacts,
    brillig::{Brillig, BrilligOptions},
    errors::{RuntimeError, SsaReport},
};
use acvm::{
    FieldElement,
    acir::{
        circuit::{
            AcirOpcodeLocation, Circuit, ErrorSelector, ExpressionWidth, OpcodeLocation,
            Program as AcirProgram, PublicInputs, brillig::BrilligBytecode,
        },
        native_types::Witness,
    },
};

use ir::instruction::ErrorType;
use noirc_errors::{
    call_stack::CallStackId,
    debug_info::{DebugFunctions, DebugInfo, DebugTypes, DebugVariables},
};

use noirc_frontend::shared::Visibility;
use noirc_frontend::{hir_def::function::FunctionSignature, monomorphization::ast::Program};
use ssa_gen::Ssa;
use tracing::{Level, span};

use crate::acir::GeneratedAcir;

mod checks;
pub mod function_builder;
pub mod interpreter;
pub mod ir;
pub(crate) mod opt;
pub mod parser;
pub mod ssa_gen;
pub(crate) mod validation;

#[derive(Debug, Clone)]
pub enum SsaLogging {
    None,
    All,
    Contains(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct SsaEvaluatorOptions {
    /// Emit debug information for the intermediate SSA IR
    pub ssa_logging: SsaLogging,

    /// Options affecting Brillig code generation.
    pub brillig_options: BrilligOptions,

    /// Pretty print benchmark times of each code generation pass
    pub print_codegen_timings: bool,

    /// Width of expressions to be used for ACIR
    pub expression_width: ExpressionWidth,

    /// Dump the unoptimized SSA to the supplied path if it exists
    pub emit_ssa: Option<PathBuf>,

    /// Skip the check for under constrained values
    pub skip_underconstrained_check: bool,

    /// Skip the missing Brillig call constraints check
    pub skip_brillig_constraints_check: bool,

    /// Enable the lookback feature of the Brillig call constraints
    /// check (prevents some rare false positives, leads to a slowdown
    /// on large rollout functions)
    pub enable_brillig_constraints_check_lookback: bool,

    /// The higher the value, the more inlined Brillig functions will be.
    pub inliner_aggressiveness: i64,

    /// Maximum accepted percentage increase in the Brillig bytecode size after unrolling loops.
    /// When `None` the size increase check is skipped altogether and any decrease in the SSA
    /// instruction count is accepted.
    pub max_bytecode_increase_percent: Option<i32>,

    /// A list of SSA pass messages to skip, for testing purposes.
    pub skip_passes: Vec<String>,
}

/// An SSA pass reified as a construct we can put into a list,
/// which facilitates equivalence testing between different
/// stages of the processing pipeline.
pub struct SsaPass<'a> {
    msg: &'static str,
    run: Box<dyn Fn(Ssa) -> Result<Ssa, RuntimeError> + 'a>,
}

impl<'a> SsaPass<'a> {
    pub fn new<F>(f: F, msg: &'static str) -> Self
    where
        F: Fn(Ssa) -> Ssa + 'a,
    {
        Self::new_try(move |ssa| Ok(f(ssa)), msg)
    }

    pub fn new_try<F>(f: F, msg: &'static str) -> Self
    where
        F: Fn(Ssa) -> Result<Ssa, RuntimeError> + 'a,
    {
        Self { msg, run: Box::new(f) }
    }

    pub fn msg(&self) -> &str {
        self.msg
    }

    pub fn run(&self, ssa: Ssa) -> Result<Ssa, RuntimeError> {
        (self.run)(ssa)
    }

    /// Follow up the pass with another one, without adding a separate message for it.
    ///
    /// This is useful for attaching cleanup passes that we don't want to appear on their
    /// own in the pipeline, because it would just increase the noise.
    pub fn and_then<F>(self, f: F) -> Self
    where
        F: Fn(Ssa) -> Ssa + 'a,
    {
        self.and_then_try(move |ssa| Ok(f(ssa)))
    }

    /// Same as `and_then` but for passes that can fail.
    pub fn and_then_try<F>(self, f: F) -> Self
    where
        F: Fn(Ssa) -> Result<Ssa, RuntimeError> + 'a,
    {
        Self {
            msg: self.msg,
            run: Box::new(move |ssa| {
                let ssa = self.run(ssa)?;
                let ssa = f(ssa)?;
                Ok(ssa)
            }),
        }
    }
}

pub struct ArtifactsAndWarnings(pub Artifacts, pub Vec<SsaReport>);

/// The default SSA optimization pipeline.
///
/// After these passes everything is ready for execution, which is
/// something we take can advantage of in the [secondary_passes].
pub fn primary_passes(options: &SsaEvaluatorOptions) -> Vec<SsaPass> {
    vec![
        SsaPass::new(Ssa::remove_unreachable_functions, "Removing Unreachable Functions"),
        SsaPass::new(Ssa::defunctionalize, "Defunctionalization"),
        SsaPass::new(Ssa::inline_simple_functions, "Inlining simple functions")
            .and_then(Ssa::remove_unreachable_functions),
        // BUG: Enabling this mem2reg causes an integration test failure in aztec-package; see:
        // https://github.com/AztecProtocol/aztec-packages/pull/11294#issuecomment-2622809518
        //SsaPass::new(Ssa::mem2reg, "Mem2Reg (1st)"),
        SsaPass::new(Ssa::remove_paired_rc, "Removing Paired rc_inc & rc_decs"),
        SsaPass::new(
            move |ssa| ssa.preprocess_functions(options.inliner_aggressiveness),
            "Preprocessing Functions",
        ),
        SsaPass::new(move |ssa| ssa.inline_functions(options.inliner_aggressiveness), "Inlining"),
        // Run mem2reg with the CFG separated into blocks
        SsaPass::new(Ssa::mem2reg, "Mem2Reg"),
        SsaPass::new(Ssa::simplify_cfg, "Simplifying"),
        SsaPass::new(Ssa::as_slice_optimization, "`as_slice` optimization")
            .and_then(Ssa::remove_unreachable_functions),
        SsaPass::new_try(
            Ssa::evaluate_static_assert_and_assert_constant,
            "`static_assert` and `assert_constant`",
        ),
        SsaPass::new(Ssa::purity_analysis, "Purity Analysis"),
        SsaPass::new(Ssa::loop_invariant_code_motion, "Loop Invariant Code Motion"),
        SsaPass::new_try(
            move |ssa| ssa.unroll_loops_iteratively(options.max_bytecode_increase_percent),
            "Unrolling",
        ),
        SsaPass::new(Ssa::simplify_cfg, "Simplifying"),
        SsaPass::new(Ssa::mem2reg, "Mem2Reg"),
        SsaPass::new(Ssa::flatten_cfg, "Flattening"),
        SsaPass::new(Ssa::remove_bit_shifts, "Removing Bit Shifts"),
        // Run mem2reg once more with the flattened CFG to catch any remaining loads/stores
        SsaPass::new(Ssa::mem2reg, "Mem2Reg"),
        // Run the inlining pass again to handle functions with `InlineType::NoPredicates`.
        // Before flattening is run, we treat functions marked with the `InlineType::NoPredicates` as an entry point.
        // This pass must come immediately following `mem2reg` as the succeeding passes
        // may create an SSA which inlining fails to handle.
        SsaPass::new(
            move |ssa| ssa.inline_functions_with_no_predicates(options.inliner_aggressiveness),
            "Inlining",
        ),
        SsaPass::new_try(Ssa::remove_if_else, "Remove IfElse"),
        SsaPass::new(Ssa::purity_analysis, "Purity Analysis"),
        SsaPass::new(Ssa::fold_constants, "Constant Folding"),
        SsaPass::new(Ssa::flatten_basic_conditionals, "Simplify conditionals for unconstrained"),
        SsaPass::new(Ssa::remove_enable_side_effects, "EnableSideEffectsIf removal"),
        SsaPass::new(Ssa::fold_constants_using_constraints, "Constraint Folding"),
        SsaPass::new_try(
            move |ssa| ssa.unroll_loops_iteratively(options.max_bytecode_increase_percent),
            "Unrolling",
        ),
        SsaPass::new(Ssa::make_constrain_not_equal_instructions, "Adding constrain not equal"),
        SsaPass::new(Ssa::check_u128_mul_overflow, "Check u128 mul overflow"),
        // Simplifying the CFG can have a positive effect on mem2reg: every time we unify with a
        // yet-to-be-visited predecessor we forget known values; less blocks mean less unification.
        SsaPass::new(Ssa::simplify_cfg, "Simplifying"),
        // We cannot run mem2reg after DIE, because it removes Store instructions.
        // We have to run it before, to give it a chance to turn Store+Load into known values.
        SsaPass::new(Ssa::mem2reg, "Mem2Reg"),
        // Removing unreachable instructions before DIE, so it gets rid of loads that mem2reg couldn't,
        // if they are unreachable and would cause the DIE post-checks to fail.
        SsaPass::new(Ssa::remove_unreachable_instructions, "Remove Unreachable Instructions")
            .and_then(Ssa::remove_unreachable_functions),
        SsaPass::new(Ssa::dead_instruction_elimination, "Dead Instruction Elimination"),
        SsaPass::new(Ssa::array_set_optimization, "Array Set Optimizations"),
        // The Brillig globals pass expected that we have the used globals map set for each function.
        // The used globals map is determined during DIE, so we should duplicate entry points before a DIE pass run.
        SsaPass::new(Ssa::brillig_entry_point_analysis, "Brillig Entry Point Analysis")
            // Remove any potentially unnecessary duplication from the Brillig entry point analysis.
            .and_then(Ssa::remove_unreachable_functions),
        SsaPass::new(Ssa::remove_truncate_after_range_check, "Removing Truncate after RangeCheck"),
        // This pass makes transformations specific to Brillig generation.
        // It must be the last pass to either alter or add new instructions before Brillig generation,
        // as other semantics in the compiler can potentially break (e.g. inserting instructions).
        // We can safely place the pass before DIE as that pass only removes instructions.
        // We also need DIE's tracking of used globals in case the array get transformations
        // end up using an existing constant from the globals space.
        // This pass might result in otherwise unused global constant becoming used,
        // because the creation of shifted index constants can reuse their IDs.
        SsaPass::new(Ssa::brillig_array_get_and_set, "Brillig Array Get and Set Optimizations"),
        // Perform another DIE pass to update the used globals after offsetting Brillig indexes.
        SsaPass::new(Ssa::dead_instruction_elimination, "Dead Instruction Elimination")
            // A function can be potentially unreachable post-DIE if all calls to that function were removed.
            .and_then(Ssa::remove_unreachable_functions),
        SsaPass::new(Ssa::checked_to_unchecked, "Checked to unchecked"),
        SsaPass::new_try(
            Ssa::verify_no_dynamic_indices_to_references,
            "Verifying no dynamic array indices to reference value elements",
        ),
    ]
}

/// The second SSA pipeline, in which we take the Brillig functions compiled after
/// the primary pipeline, and execute the ones with all-constant arguments,
/// to replace the calls with the return value.
pub fn secondary_passes(brillig: &Brillig) -> Vec<SsaPass> {
    vec![
        SsaPass::new(move |ssa| ssa.fold_constants_with_brillig(brillig), "Inlining Brillig Calls"),
        SsaPass::new(Ssa::remove_unreachable_instructions, "Remove Unreachable Instructions")
            // It could happen that we inlined all calls to a given brillig function.
            // In that case it's unused so we can remove it. This is what we check next.
            .and_then(Ssa::remove_unreachable_functions),
        SsaPass::new(Ssa::dead_instruction_elimination_acir, "Dead Instruction Elimination - ACIR"),
    ]
}

/// For testing purposes we want a list of the minimum number of SSA passes that should
/// return the same result as the full pipeline.
///
/// Due to it being minimal, it can only be executed with the Brillig VM; the ACIR runtime
/// would for example require unrolling loops, which we want to avoid to keep the SSA as
/// close to the initial state as possible.
///
/// In the future, we can potentially execute the actual initial version using the SSA interpreter.
pub fn minimal_passes() -> Vec<SsaPass<'static>> {
    vec![
        // We need to get rid of function pointer parameters, otherwise they cause panic in Brillig generation.
        SsaPass::new(Ssa::defunctionalize, "Defunctionalization"),
        // Even the initial SSA generation can result in optimizations that leave a function
        // which was called in the AST not being called in the SSA. Such functions would cause
        // panics later, when we are looking for global allocations.
        SsaPass::new(Ssa::remove_unreachable_functions, "Removing Unreachable Functions"),
        // We need to add an offset to constant array indices in Brillig.
        // This can change which globals are used, because constant creation might result
        // in the (re)use of otherwise unused global values.
        SsaPass::new(Ssa::brillig_array_get_and_set, "Brillig Array Get and Set Optimizations"),
        // We need a DIE pass to populate `used_globals`, otherwise it will panic later.
        SsaPass::new(Ssa::dead_instruction_elimination, "Dead Instruction Elimination"),
    ]
}

/// Optimize the given SsaBuilder by converting it into SSA
/// form and performing optimizations there. When finished,
/// convert the final SSA into an ACIR program and return it.
/// An ACIR program is made up of both ACIR functions
/// and Brillig functions for unconstrained execution.
///
/// The `primary` SSA passes are applied on the initial SSA.
/// Then we compile the Brillig functions, and use the output
/// to run a `secondary` pass, which can use the Brillig
/// artifacts to do constant folding.
///
/// See the [primary_passes] and [secondary_passes] for
/// the default implementations.
pub fn optimize_ssa_builder_into_acir<S>(
    builder: SsaBuilder,
    options: &SsaEvaluatorOptions,
    primary: &[SsaPass],
    secondary: S,
) -> Result<ArtifactsAndWarnings, RuntimeError>
where
    S: for<'b> Fn(&'b Brillig) -> Vec<SsaPass<'b>>,
{
    let ssa_gen_span = span!(Level::TRACE, "ssa_generation");
    let ssa_gen_span_guard = ssa_gen_span.enter();
    let mut builder = builder.with_skip_passes(options.skip_passes.clone()).run_passes(primary)?;
    let passed = std::mem::take(&mut builder.passed);
    let files = builder.files;
    let mut ssa = builder.finish();

    let mut ssa_level_warnings = vec![];
    drop(ssa_gen_span_guard);

    let used_globals_map = std::mem::take(&mut ssa.used_globals);
    let brillig = time("SSA to Brillig", options.print_codegen_timings, || {
        ssa.to_brillig_with_globals(&options.brillig_options, used_globals_map)
    });

    let ssa_gen_span = span!(Level::TRACE, "ssa_generation");
    let ssa_gen_span_guard = ssa_gen_span.enter();

    let mut ssa = SsaBuilder::from_ssa(
        ssa,
        options.ssa_logging.clone(),
        options.print_codegen_timings,
        files,
    )
    .with_passed(passed)
    .with_skip_passes(options.skip_passes.clone())
    .run_passes(&secondary(&brillig))?
    .finish();

    if !options.skip_underconstrained_check {
        ssa_level_warnings.extend(time(
            "After Check for Underconstrained Values",
            options.print_codegen_timings,
            || ssa.check_for_underconstrained_values(),
        ));
    }

    if !options.skip_brillig_constraints_check {
        ssa_level_warnings.extend(time(
            "After Check for Missing Brillig Call Constraints",
            options.print_codegen_timings,
            || {
                ssa.check_for_missing_brillig_constraints(
                    options.enable_brillig_constraints_check_lookback,
                )
            },
        ));
    };

    drop(ssa_gen_span_guard);
    let artifacts = time("SSA to ACIR", options.print_codegen_timings, || {
        ssa.into_acir(&brillig, &options.brillig_options, options.expression_width)
    })?;

    Ok(ArtifactsAndWarnings(artifacts, ssa_level_warnings))
}

/// Optimize the given program by converting it into SSA
/// form and performing optimizations there. When finished,
/// convert the final SSA into an ACIR program and return it.
/// An ACIR program is made up of both ACIR functions
/// and Brillig functions for unconstrained execution.
///
/// The `primary` SSA passes are applied on the initial SSA.
/// Then we compile the Brillig functions, and use the output
/// to run a `secondary` pass, which can use the Brillig
/// artifacts to do constant folding.
///
/// See the [primary_passes] and [secondary_passes] for
/// the default implementations.
pub fn optimize_into_acir<S>(
    program: Program,
    options: &SsaEvaluatorOptions,
    primary: &[SsaPass],
    secondary: S,
    files: Option<&fm::FileManager>,
) -> Result<ArtifactsAndWarnings, RuntimeError>
where
    S: for<'b> Fn(&'b Brillig) -> Vec<SsaPass<'b>>,
{
    let builder = SsaBuilder::from_program(
        program,
        options.ssa_logging.clone(),
        options.print_codegen_timings,
        &options.emit_ssa,
        files,
    )?;

    optimize_ssa_builder_into_acir(builder, options, primary, secondary)
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
    pub brillig_names: Vec<String>,
    pub error_types: BTreeMap<ErrorSelector, ErrorType>,
}

impl SsaProgramArtifact {
    pub fn new(
        unconstrained_functions: Vec<BrilligBytecode<FieldElement>>,
        error_types: BTreeMap<ErrorSelector, ErrorType>,
    ) -> Self {
        let program = AcirProgram { functions: Vec::default(), unconstrained_functions };
        Self {
            program,
            debug: Vec::default(),
            warnings: Vec::default(),
            main_input_witnesses: Vec::default(),
            main_return_witnesses: Vec::default(),
            names: Vec::default(),
            brillig_names: Vec::default(),
            error_types,
        }
    }

    pub fn add_circuit(&mut self, mut circuit_artifact: SsaCircuitArtifact, is_main: bool) {
        self.program.functions.push(circuit_artifact.circuit);
        self.debug.push(circuit_artifact.debug_info);
        self.warnings.append(&mut circuit_artifact.warnings);
        if is_main {
            self.main_input_witnesses = circuit_artifact.input_witnesses;
            self.main_return_witnesses = circuit_artifact.return_witnesses;
        }
        self.names.push(circuit_artifact.name);
        // Acir and brillig both generate new error types, so we need to merge them
        // With the ones found during ssa generation.
        self.error_types.extend(circuit_artifact.error_types);
    }

    fn add_warnings(&mut self, mut warnings: Vec<SsaReport>) {
        self.warnings.append(&mut warnings);
    }
}

/// Compiles the [`Program`] into [`ACIR`][acvm::acir::circuit::Program].
///
/// The output ACIR is backend-agnostic and so must go through a transformation pass before usage in proof generation.
#[tracing::instrument(level = "trace", skip_all)]
pub fn create_program(
    program: Program,
    options: &SsaEvaluatorOptions,
    files: Option<&fm::FileManager>,
) -> Result<SsaProgramArtifact, RuntimeError> {
    create_program_with_passes(program, options, &primary_passes(options), secondary_passes, files)
}

/// Compiles the [`Program`] into [`ACIR`][acvm::acir::circuit::Program] using the minimum amount of SSA passes.
///
/// This is intended for testing purposes, and currently requires the program to be compiled for Brillig.
/// It is not added to the `SsaEvaluatorOptions` to avoid ambiguity when calling `create_program_with_passes` directly.
#[tracing::instrument(level = "trace", skip_all)]
pub fn create_program_with_minimal_passes(
    program: Program,
    options: &SsaEvaluatorOptions,
    files: &fm::FileManager,
) -> Result<SsaProgramArtifact, RuntimeError> {
    for func in &program.functions {
        assert!(
            func.unconstrained,
            "The minimum SSA pipeline only works with Brillig: '{}' needs to be unconstrained",
            func.name
        );
    }
    create_program_with_passes(program, options, &minimal_passes(), |_| vec![], Some(files))
}

/// Compiles the [`Program`] into [`ACIR`][acvm::acir::circuit::Program] by running it through
/// `primary` and `secondary` SSA passes.
#[tracing::instrument(level = "trace", skip_all)]
pub fn create_program_with_passes<S>(
    program: Program,
    options: &SsaEvaluatorOptions,
    primary: &[SsaPass],
    secondary: S,
    files: Option<&fm::FileManager>,
) -> Result<SsaProgramArtifact, RuntimeError>
where
    S: for<'b> Fn(&'b Brillig) -> Vec<SsaPass<'b>>,
{
    let debug_variables = program.debug_variables.clone();
    let debug_types = program.debug_types.clone();
    let debug_functions = program.debug_functions.clone();

    let func_sigs = program.function_signatures.clone();

    let ArtifactsAndWarnings(
        (generated_acirs, generated_brillig, brillig_function_names, error_types),
        ssa_level_warnings,
    ) = optimize_into_acir(program, options, primary, secondary, files)?;

    assert_eq!(
        generated_acirs.len(),
        func_sigs.len(),
        "The generated ACIRs should match the supplied function signatures"
    );

    let error_types = error_types
        .into_iter()
        .map(|(selector, hir_type)| (selector, ErrorType::Dynamic(hir_type)))
        .collect();

    let mut program_artifact = SsaProgramArtifact::new(generated_brillig, error_types);

    // Add warnings collected at the Ssa stage
    program_artifact.add_warnings(ssa_level_warnings);
    // For setting up the ABI we need separately specify main's input and return witnesses
    let mut is_main = true;
    for (acir, func_sig) in generated_acirs.into_iter().zip(func_sigs) {
        let circuit_artifact = convert_generated_acir_into_circuit(
            acir,
            func_sig,
            // TODO: get rid of these clones
            debug_variables.clone(),
            debug_functions.clone(),
            debug_types.clone(),
        );
        program_artifact.add_circuit(circuit_artifact, is_main);
        is_main = false;
    }
    program_artifact.brillig_names = brillig_function_names;

    Ok(program_artifact)
}

pub struct SsaCircuitArtifact {
    pub name: String,
    pub circuit: Circuit<FieldElement>,
    pub debug_info: DebugInfo,
    pub warnings: Vec<SsaReport>,
    pub input_witnesses: Vec<Witness>,
    pub return_witnesses: Vec<Witness>,
    pub error_types: BTreeMap<ErrorSelector, ErrorType>,
}

pub fn convert_generated_acir_into_circuit(
    mut generated_acir: GeneratedAcir<FieldElement>,
    func_sig: FunctionSignature,
    debug_variables: DebugVariables,
    debug_functions: DebugFunctions,
    debug_types: DebugTypes,
) -> SsaCircuitArtifact {
    let opcodes = generated_acir.take_opcodes();
    let current_witness_index = generated_acir.current_witness_index().0;
    let GeneratedAcir {
        return_witnesses,
        location_map,
        brillig_locations,
        input_witnesses,
        assertion_payloads: assert_messages,
        warnings,
        name,
        brillig_procedure_locs,
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
    };
    let acir_location_map: BTreeMap<AcirOpcodeLocation, CallStackId> = location_map
        .iter()
        .map(|(k, v)| match k {
            OpcodeLocation::Acir(index) => (AcirOpcodeLocation::new(*index), *v),
            OpcodeLocation::Brillig { .. } => unreachable!("Expected ACIR opcode"),
        })
        .collect();
    let location_tree = generated_acir.call_stacks.to_location_tree();
    let mut debug_info = DebugInfo::new(
        brillig_locations,
        acir_location_map,
        location_tree,
        debug_variables,
        debug_functions,
        debug_types,
        brillig_procedure_locs,
    );

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
        error_types: generated_acir.error_types,
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
        .map(|(pattern, typ, visibility)| {
            let num_field_elements_needed = typ.field_count(&pattern.location()) as usize;
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
pub struct SsaBuilder<'local> {
    /// The SSA being built; it is the input and the output of every pass ran by the builder.
    pub ssa: Ssa,
    /// Options to control which SSA passes to print.
    pub ssa_logging: SsaLogging,
    /// Whether to print the amount of time it took to run individual SSA passes.
    pub print_codegen_timings: bool,
    /// Counters indexed by the message in the SSA pass, so we can distinguish between multiple
    /// runs of the same pass in the printed messages.
    pub passed: HashMap<String, usize>,
    /// List of SSA pass message fragments that we want to skip, for testing purposes.
    pub skip_passes: Vec<String>,

    /// Providing a file manager is optional - if provided it can be used to print source
    /// locations along with each ssa instructions when debugging.
    pub files: Option<&'local fm::FileManager>,
}

impl<'local> SsaBuilder<'local> {
    pub fn from_program(
        program: Program,
        ssa_logging: SsaLogging,
        print_codegen_timings: bool,
        emit_ssa: &Option<PathBuf>,
        files: Option<&'local fm::FileManager>,
    ) -> Result<Self, RuntimeError> {
        let ssa = ssa_gen::generate_ssa(program)?;
        if let Some(emit_ssa) = emit_ssa {
            let mut emit_ssa_dir = emit_ssa.clone();
            // We expect the full package artifact path to be passed in here,
            // and attempt to create the target directory if it does not exist.
            emit_ssa_dir.pop();
            create_named_dir(emit_ssa_dir.as_ref(), "target");
            let ssa_path = emit_ssa.with_extension("ssa.json");
            write_to_file(&serde_json::to_vec(&ssa).unwrap(), &ssa_path);
        }
        Ok(Self::from_ssa(ssa, ssa_logging, print_codegen_timings, files).print("Initial SSA"))
    }

    pub fn from_ssa(
        ssa: Ssa,
        ssa_logging: SsaLogging,
        print_codegen_timings: bool,
        files: Option<&'local fm::FileManager>,
    ) -> Self {
        Self {
            ssa_logging,
            print_codegen_timings,
            ssa,
            files,
            passed: Default::default(),
            skip_passes: Default::default(),
        }
    }

    pub fn with_passed(mut self, passed: HashMap<String, usize>) -> Self {
        self.passed = passed;
        self
    }

    pub fn with_skip_passes(mut self, skip_passes: Vec<String>) -> Self {
        self.skip_passes = skip_passes;
        self
    }

    pub fn finish(self) -> Ssa {
        self.ssa.generate_entry_point_index()
    }

    /// Run a list of SSA passes.
    fn run_passes(mut self, passes: &[SsaPass]) -> Result<Self, RuntimeError> {
        for pass in passes {
            self = self.try_run_pass(|ssa| pass.run(ssa), pass.msg)?;
        }
        Ok(self)
    }

    /// Runs the given SSA pass and prints the SSA afterward if `print_ssa_passes` is true.
    #[allow(dead_code)]
    fn run_pass<F>(mut self, pass: F, msg: &str) -> Self
    where
        F: FnOnce(Ssa) -> Ssa,
    {
        self.ssa = time(msg, self.print_codegen_timings, || pass(self.ssa));
        self.print(msg)
    }

    /// The same as `run_pass` but for passes that may fail
    fn try_run_pass<F>(mut self, pass: F, msg: &str) -> Result<Self, RuntimeError>
    where
        F: FnOnce(Ssa) -> Result<Ssa, RuntimeError>,
    {
        // Count the number of times we have seen this message.
        let cnt = *self.passed.entry(msg.to_string()).and_modify(|cnt| *cnt += 1).or_insert(1);
        let step = self.passed.values().sum::<usize>();
        let msg = format!("{msg} ({cnt}) (step {step})");

        // See if we should skip this pass, including the count, so we can skip the n-th occurrence of a step.
        let skip = self.skip_passes.iter().any(|s| msg.contains(s));

        if !skip {
            self.ssa = time(&msg, self.print_codegen_timings, || pass(self.ssa))?;
            Ok(self.print(&msg))
        } else {
            Ok(self)
        }
    }

    fn print(mut self, msg: &str) -> Self {
        // Always normalize if we are going to print at least one of the passes
        if !matches!(self.ssa_logging, SsaLogging::None) {
            self.ssa.normalize_ids();
        }

        let print_ssa_pass = match &self.ssa_logging {
            SsaLogging::None => false,
            SsaLogging::All => true,
            SsaLogging::Contains(strings) => strings.iter().any(|string| {
                let string = string.to_lowercase();
                let string = string.strip_prefix("after ").unwrap_or(&string);
                let string = string.strip_suffix(':').unwrap_or(string);
                msg.to_lowercase().contains(string)
            }),
        };

        if print_ssa_pass {
            println!("After {msg}:\n{}", self.ssa.print_with(self.files));
        }
        self
    }
}

fn create_named_dir(named_dir: &Path, name: &str) -> PathBuf {
    std::fs::create_dir_all(named_dir)
        .unwrap_or_else(|_| panic!("could not create the `{name}` directory"));

    PathBuf::from(named_dir)
}

fn write_to_file(bytes: &[u8], path: &Path) {
    let display = path.display();

    let mut file = match File::create(path) {
        Err(why) => panic!("couldn't create {display}: {why}"),
        Ok(file) => file,
    };

    if let Err(why) = file.write_all(bytes) {
        panic!("couldn't write to {display}: {why}");
    }
}
