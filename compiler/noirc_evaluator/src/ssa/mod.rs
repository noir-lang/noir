//! SSA stands for Single Static Assignment
//! The IR presented in this module will already
//! be in SSA form and will be used to apply
//! conventional optimizations like Common Subexpression
//! elimination and constant folding.
//!
//! This module heavily borrows from Cranelift

use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use crate::{
    acir::ssa::Artifacts,
    brillig::BrilligOptions,
    errors::{RuntimeError, SsaReport},
};
use acvm::{
    FieldElement,
    acir::{
        circuit::{AcirOpcodeLocation, Circuit, OpcodeLocation, PublicInputs},
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

pub use artifact::{SsaCircuitArtifact, SsaProgramArtifact};
use builder::time;
pub use builder::{SsaBuilder, SsaPass};

mod artifact;
mod builder;
mod checks;
pub mod function_builder;
pub mod interpreter;
pub mod ir;
pub mod opt;
pub mod parser;
pub mod ssa_gen;
pub(crate) mod validation;
mod visit_once_deque;
mod visit_once_priority_queue;

#[derive(Debug, Clone)]
pub enum SsaLogging {
    None,
    All,
    Contains(Vec<String>),
}

impl SsaLogging {
    /// Check if an SSA pass should be printed.
    pub fn matches(&self, msg: &str) -> bool {
        match self {
            SsaLogging::None => false,
            SsaLogging::All => true,
            SsaLogging::Contains(strings) => strings.iter().any(|string| {
                let string = string.to_lowercase();
                let string = string.strip_prefix("after ").unwrap_or(&string);
                let string = string.strip_suffix(':').unwrap_or(string);
                msg.to_lowercase().contains(string)
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SsaEvaluatorOptions {
    /// Emit debug information for the intermediate SSA IR
    pub ssa_logging: SsaLogging,

    /// Options affecting Brillig code generation.
    pub brillig_options: BrilligOptions,

    /// Pretty print benchmark times of each code generation pass
    pub print_codegen_timings: bool,

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

    /// Maximum number iterations to do in constant folding, as long as new values are hoisted.
    pub constant_folding_max_iter: usize,

    /// The higher the value, the more Brillig functions will be set to always be inlined.
    pub small_function_max_instruction: usize,

    /// Maximum accepted percentage increase in the Brillig bytecode size after unrolling loops.
    /// When `None` the size increase check is skipped altogether and any decrease in the SSA
    /// instruction count is accepted.
    pub max_bytecode_increase_percent: Option<i32>,

    /// A list of SSA pass messages to skip, for testing purposes.
    pub skip_passes: Vec<String>,
}

pub struct ArtifactsAndWarnings(pub Artifacts, pub Vec<SsaReport>);

/// The default SSA optimization pipeline.
pub fn primary_passes(options: &SsaEvaluatorOptions) -> Vec<SsaPass<'_>> {
    vec![
        SsaPass::new(Ssa::expand_signed_checks, "expand signed checks"),
        SsaPass::new(Ssa::remove_unreachable_functions, "Removing Unreachable Functions"),
        SsaPass::new(Ssa::defunctionalize, "Defunctionalization"),
        SsaPass::new_try(Ssa::inline_simple_functions, "Inlining simple functions")
            .and_then(Ssa::remove_unreachable_functions),
        // BUG: Enabling this mem2reg causes test failures in aztec-nr; specifically `state_vars::private_mutable::test::initialize_and_get_pending`
        // SsaPass::new(Ssa::mem2reg, "Mem2Reg"),
        SsaPass::new(Ssa::remove_paired_rc, "Removing Paired rc_inc & rc_decs"),
        SsaPass::new(Ssa::purity_analysis, "Purity Analysis"),
        SsaPass::new_try(
            move |ssa| {
                ssa.preprocess_functions(
                    options.inliner_aggressiveness,
                    options.small_function_max_instruction,
                )
            },
            "Preprocessing Functions",
        ),
        SsaPass::new_try(
            move |ssa| {
                ssa.inline_functions(
                    options.inliner_aggressiveness,
                    options.small_function_max_instruction,
                )
            },
            "Inlining",
        ),
        // Run mem2reg with the CFG separated into blocks
        SsaPass::new(Ssa::mem2reg, "Mem2Reg"),
        // Running DIE here might remove some unused instructions mem2reg could not eliminate.
        SsaPass::new(
            Ssa::dead_instruction_elimination_pre_flattening,
            "Dead Instruction Elimination",
        ),
        SsaPass::new(Ssa::simplify_cfg, "Simplifying"),
        SsaPass::new(Ssa::as_list_optimization, "`as_list` optimization")
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
        SsaPass::new(Ssa::remove_bit_shifts, "Removing Bit Shifts"),
        // Expand signed lt/div/mod after "Removing Bit Shifts" because that pass might
        // introduce signed divisions.
        SsaPass::new(Ssa::expand_signed_math, "Expand signed math"),
        SsaPass::new(Ssa::simplify_cfg, "Simplifying"),
        SsaPass::new(Ssa::flatten_cfg, "Flattening"),
        // Run mem2reg once more with the flattened CFG to catch any remaining loads/stores,
        // then try to free memory before inlining, which involves copying a instructions.
        SsaPass::new(Ssa::mem2reg, "Mem2Reg").and_then(Ssa::remove_unused_instructions),
        // Run the inlining pass again to handle functions with `InlineType::NoPredicates`.
        // Before flattening is run, we treat functions marked with the `InlineType::NoPredicates` as an entry point.
        // This pass must come immediately following `mem2reg` as the succeeding passes
        // may create an SSA which inlining fails to handle.
        SsaPass::new_try(
            move |ssa| {
                ssa.inline_functions_with_no_predicates(
                    options.inliner_aggressiveness,
                    options.small_function_max_instruction,
                )
            },
            "Inlining",
        ),
        SsaPass::new_try(Ssa::remove_if_else, "Remove IfElse"),
        SsaPass::new(Ssa::purity_analysis, "Purity Analysis"),
        SsaPass::new(
            |ssa| ssa.fold_constants(options.constant_folding_max_iter),
            "Constant Folding",
        ),
        SsaPass::new(Ssa::flatten_basic_conditionals, "Simplify conditionals for unconstrained"),
        SsaPass::new(Ssa::remove_enable_side_effects, "EnableSideEffectsIf removal"),
        SsaPass::new(
            |ssa| ssa.fold_constants_using_constraints(options.constant_folding_max_iter),
            "Constant Folding using constraints",
        ),
        SsaPass::new_try(
            move |ssa| ssa.unroll_loops_iteratively(options.max_bytecode_increase_percent),
            "Unrolling",
        ),
        SsaPass::new(Ssa::make_constrain_not_equal, "Adding constrain not equal"),
        SsaPass::new(Ssa::check_u128_mul_overflow, "Check u128 mul overflow"),
        // Simplifying the CFG can have a positive effect on mem2reg: every time we unify with a
        // yet-to-be-visited predecessor we forget known values; less blocks mean less unification.
        SsaPass::new(Ssa::simplify_cfg, "Simplifying"),
        // Removing unreachable instructions before mem2reg, which may result in some default Store
        // instructions being added, which it can pair up with Loads. If we ran it after it,
        // then DIE would just remove the Stores, leaving the Loads dangling.
        // This has to be done after flattening, as it destroys the CFG by removing terminators.
        SsaPass::new(Ssa::remove_unreachable_instructions, "Remove Unreachable Instructions")
            .and_then(Ssa::remove_unreachable_functions),
        // We cannot run mem2reg after DIE, because it removes Store instructions.
        // We have to run it before, to give it a chance to turn Store+Load into known values.
        SsaPass::new(Ssa::mem2reg, "Mem2Reg"),
        SsaPass::new(Ssa::dead_instruction_elimination, "Dead Instruction Elimination"),
        SsaPass::new(Ssa::brillig_entry_point_analysis, "Brillig Entry Point Analysis")
            // Remove any potentially unnecessary duplication from the Brillig entry point analysis.
            .and_then(Ssa::remove_unreachable_functions),
        SsaPass::new(Ssa::remove_truncate_after_range_check, "Removing Truncate after RangeCheck"),
        SsaPass::new(Ssa::checked_to_unchecked, "Checked to unchecked"),
        SsaPass::new(
            |ssa| ssa.fold_constants_using_constraints(options.constant_folding_max_iter),
            "Constant Folding using constraints",
        ),
        SsaPass::new(
            |ssa| ssa.fold_constants_with_brillig(options.constant_folding_max_iter),
            "Inlining Brillig Calls",
        ),
        SsaPass::new(Ssa::remove_unreachable_instructions, "Remove Unreachable Instructions")
            .and_then(Ssa::remove_unreachable_functions),
        SsaPass::new(Ssa::dead_instruction_elimination, "Dead Instruction Elimination")
            // A function can be potentially unreachable post-DIE if all calls to that function were removed.
            .and_then(Ssa::remove_unreachable_functions),
        SsaPass::new_try(
            Ssa::verify_no_dynamic_indices_to_references,
            "Verifying no dynamic array indices to reference value elements",
        ),
        SsaPass::new(Ssa::array_set_optimization, "Array Set Optimizations").and_then(|ssa| {
            // Deferred sanity checks that don't modify the SSA, just panic if we have something unexpected
            // that we don't know how to attribute to a concrete error with the Noir code.
            ssa.dead_instruction_elimination_post_check(true);
            ssa
        }),
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
        // Signed integer operations need to be expanded in order to have the appropriate overflow checks applied.
        SsaPass::new(Ssa::expand_signed_checks, "expand signed checks"),
        // We need to get rid of function pointer parameters, otherwise they cause panic in Brillig generation.
        SsaPass::new(Ssa::defunctionalize, "Defunctionalization"),
        // Even the initial SSA generation can result in optimizations that leave a function
        // which was called in the AST not being called in the SSA. Such functions would cause
        // panics later, when we are looking for global allocations.
        SsaPass::new(Ssa::remove_unreachable_functions, "Removing Unreachable Functions"),
    ]
}

/// Optimize the given SsaBuilder by converting it into SSA
/// form and performing optimizations there. When finished,
/// convert the final SSA into an ACIR program and return it.
/// An ACIR program is made up of both ACIR functions
/// and Brillig functions for unconstrained execution.
pub fn optimize_ssa_builder_into_acir(
    builder: SsaBuilder,
    options: &SsaEvaluatorOptions,
    passes: &[SsaPass],
) -> Result<ArtifactsAndWarnings, RuntimeError> {
    let ssa_gen_span = span!(Level::TRACE, "ssa_generation");
    let ssa_gen_span_guard = ssa_gen_span.enter();
    let builder = builder.with_skip_passes(options.skip_passes.clone()).run_passes(passes)?;

    drop(ssa_gen_span_guard);

    // Shift the array offsets in Brillig functions to benefit from the constant allocation logic,
    // but do this outside the normal SSA passes, so we don't need to make the SSA interpreter and
    // every other pass aware of the possibility that indexes are shifted, which could result in
    // them conceptually not aligning with the positions of complex item types for example.
    // This can change which globals are used, because constant creation might result
    // in the (re)use of otherwise unused global values.
    // It must be the last pass to either alter or add new instructions before Brillig generation,
    // as other semantics in the compiler can potentially break (e.g. inserting instructions).
    // Running it through the builder so it can be printed, for transparency.
    let builder = builder.run_passes(&[SsaPass::new(
        Ssa::brillig_array_get_and_set,
        "Brillig Array Get and Set Optimizations",
    )])?;

    let brillig = time("SSA to Brillig", options.print_codegen_timings, || {
        builder.ssa().to_brillig(&options.brillig_options)
    });

    let ssa_gen_span_guard = ssa_gen_span.enter();

    let mut ssa = builder.finish();
    let mut ssa_level_warnings = vec![];
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
        ssa.into_acir(&brillig, &options.brillig_options)
    })?;

    Ok(ArtifactsAndWarnings(artifacts, ssa_level_warnings))
}

/// Optimize the given program by converting it into SSA
/// form and performing optimizations there. When finished,
/// convert the final SSA into an ACIR program and return it.
/// An ACIR program is made up of both ACIR functions
/// and Brillig functions for unconstrained execution.
pub fn optimize_into_acir(
    program: Program,
    options: &SsaEvaluatorOptions,
    passes: &[SsaPass],
    files: Option<&fm::FileManager>,
) -> Result<ArtifactsAndWarnings, RuntimeError> {
    let builder = SsaBuilder::from_program(
        program,
        options.ssa_logging.clone(),
        options.print_codegen_timings,
        &options.emit_ssa,
        files,
    )?;

    optimize_ssa_builder_into_acir(builder, options, passes)
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
    create_program_with_passes(program, options, &primary_passes(options), files)
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
            "The minimal SSA pipeline only works with Brillig: '{}' needs to be unconstrained",
            func.name
        );
    }
    create_program_with_passes(program, options, &minimal_passes(), Some(files))
}

/// Compiles the [`Program`] into [`ACIR`][acvm::acir::circuit::Program] by running it through SSA `passes`.
#[tracing::instrument(level = "trace", skip_all)]
pub fn create_program_with_passes(
    program: Program,
    options: &SsaEvaluatorOptions,
    passes: &[SsaPass],
    files: Option<&fm::FileManager>,
) -> Result<SsaProgramArtifact, RuntimeError> {
    let debug_variables = program.debug_variables.clone();
    let debug_types = program.debug_types.clone();
    let debug_functions = program.debug_functions.clone();

    let arg_size_and_visibilities: Vec<Vec<(u32, Visibility)>> =
        program.function_signatures.iter().map(resolve_function_signature).collect();

    let artifacts = optimize_into_acir(program, options, passes, files)?;

    Ok(combine_artifacts(
        artifacts,
        &arg_size_and_visibilities,
        debug_variables,
        debug_functions,
        debug_types,
    ))
}

pub fn combine_artifacts(
    artifacts: ArtifactsAndWarnings,
    arg_size_and_visibilities: &[Vec<(u32, Visibility)>],
    debug_variables: DebugVariables,
    debug_functions: DebugFunctions,
    debug_types: DebugTypes,
) -> SsaProgramArtifact {
    let ArtifactsAndWarnings((generated_acirs, generated_brillig, error_types), ssa_level_warnings) =
        artifacts;

    assert_eq!(
        generated_acirs.len(),
        arg_size_and_visibilities.len(),
        "The generated ACIRs should match the supplied function signatures"
    );
    let functions: Vec<SsaCircuitArtifact> = generated_acirs
        .into_iter()
        .zip(arg_size_and_visibilities)
        .map(|(acir, arg_size_and_visibility)| {
            convert_generated_acir_into_circuit(
                acir,
                arg_size_and_visibility,
                // TODO: get rid of these clones
                debug_variables.clone(),
                debug_functions.clone(),
                debug_types.clone(),
            )
        })
        .collect();

    let error_types = error_types
        .into_iter()
        .map(|(selector, hir_type)| (selector, ErrorType::Dynamic(hir_type)))
        .collect();

    SsaProgramArtifact::new(functions, generated_brillig, error_types, ssa_level_warnings)
}

fn resolve_function_signature(func_sig: &FunctionSignature) -> Vec<(u32, Visibility)> {
    func_sig
        .0
        .iter()
        .map(|(pattern, typ, visibility)| (typ.field_count(&pattern.location()), *visibility))
        .collect()
}

pub fn convert_generated_acir_into_circuit(
    mut generated_acir: GeneratedAcir<FieldElement>,
    arg_size_and_visibility: &[(u32, Visibility)],
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
        split_public_and_private_inputs(arg_size_and_visibility, &input_witnesses);

    let public_parameters = PublicInputs(public_parameter_witnesses);
    let return_values = PublicInputs(return_witnesses.iter().copied().collect());

    let circuit = Circuit {
        function_name: name.clone(),
        current_witness_index,
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

    // We don't have Brillig info available here yet.
    let brillig_side_effects = BTreeMap::new();
    // Perform any ACIR-level optimizations
    let (optimized_circuit, transformation_map) =
        acvm::compiler::optimize(circuit, &brillig_side_effects);
    debug_info.update_acir(transformation_map);

    SsaCircuitArtifact {
        name,
        circuit: optimized_circuit,
        debug_info,
        warnings,
        error_types: generated_acir.error_types,
    }
}

// Takes each function argument and partitions the circuit's inputs witnesses according to its visibility.
fn split_public_and_private_inputs(
    argument_sizes: &[(u32, Visibility)],
    input_witnesses: &[Witness],
) -> (BTreeSet<Witness>, BTreeSet<Witness>) {
    let mut idx = 0_usize;
    if input_witnesses.is_empty() {
        return (BTreeSet::new(), BTreeSet::new());
    }

    argument_sizes
        .iter()
        .map(|(arg_size, visibility)| {
            let arg_size = *arg_size as usize;
            let witnesses = input_witnesses[idx..idx + arg_size].to_vec();
            idx += arg_size;
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
