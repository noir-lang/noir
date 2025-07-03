use acvm::acir::circuit::PublicInputs;
use acvm::{
    FieldElement,
    acir::circuit::{AcirOpcodeLocation, Circuit, ExpressionWidth, OpcodeLocation},
};
use noirc_abi::Abi;
use noirc_driver::{CompileError, CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING};
use noirc_errors::{
    call_stack::{CallStack, CallStackId},
    debug_info::{DebugFunctions, DebugInfo, DebugTypes, DebugVariables},
};
use noirc_evaluator::{
    acir::GeneratedAcir,
    brillig::BrilligOptions,
    errors::{InternalError, RuntimeError},
    ssa::{
        ArtifactsAndWarnings, SsaBuilder, SsaCircuitArtifact, SsaEvaluatorOptions, SsaLogging,
        SsaProgramArtifact,
        ir::instruction::ErrorType,
        optimize_ssa_builder_into_acir, primary_passes, secondary_passes,
        ssa_gen::{Ssa, validate_ssa},
    },
};
use std::collections::{BTreeMap, BTreeSet};
use std::panic::AssertUnwindSafe;

/// Optimizes the given SSA into ACIR
pub fn optimize_ssa_into_acir(
    ssa: Ssa,
    options: SsaEvaluatorOptions,
) -> Result<ArtifactsAndWarnings, RuntimeError> {
    let previous_hook = std::panic::take_hook();
    let panic_message = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let hook_message = panic_message.clone();

    std::panic::set_hook(Box::new(move |panic_info| {
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            format!("Panic: {}", s)
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            format!("Panic: {}", s)
        } else {
            format!("Unknown panic: {:?}", panic_info)
        };

        if let Some(location) = panic_info.location() {
            let loc_info = format!(" at {}:{}", location.file(), location.line());
            *hook_message.lock().unwrap() = message + &loc_info;
        } else {
            *hook_message.lock().unwrap() = message;
        }
    }));
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        validate_ssa(&ssa);
        let builder = SsaBuilder::from_ssa(
            ssa,
            options.ssa_logging.clone(),
            options.print_codegen_timings,
            None,
        );
        optimize_ssa_builder_into_acir(
            builder,
            &options,
            &primary_passes(&options),
            secondary_passes,
        )
    }));
    std::panic::set_hook(previous_hook);
    match result {
        Ok(r) => r,
        Err(_) => {
            let error_msg = panic_message.lock().unwrap().clone();
            Err(RuntimeError::InternalError(InternalError::General {
                message: format!("Panic occurred: {}", error_msg),
                call_stack: CallStack::default(),
            }))
        }
    }
}

/// Converts the generated ACIR into a circuit artifact
/// its taken from noirc_evaluator::ssa::convert_generated_acir_into_circuit,
/// but modified not to use signature
/// in initial function signature used to split public, private and return witnesses
/// but now we don't need it, because we don't have any public inputs in SSA context,
/// so we just use all witnesses as private inputs and return witnesses
fn convert_generated_acir_into_circuit_without_signature(
    mut generated_acir: GeneratedAcir<FieldElement>,
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

    let private_parameters = BTreeSet::from_iter(input_witnesses.iter().copied());
    let public_parameters = PublicInputs(BTreeSet::new());
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

/// Creates a program artifact from the given FunctionBuilder
/// its taken from noirc_evaluator::ssa::create_program, but modified to accept FunctionBuilder
fn create_program(artifacts: ArtifactsAndWarnings) -> Result<SsaProgramArtifact, RuntimeError> {
    let ArtifactsAndWarnings(
        (generated_acirs, generated_brillig, brillig_function_names, error_types),
        _ssa_level_warnings,
    ) = artifacts;

    let error_types = error_types
        .into_iter()
        .map(|(selector, hir_type)| (selector, ErrorType::Dynamic(hir_type)))
        .collect();
    let mut program_artifact = SsaProgramArtifact::new(generated_brillig, error_types);
    let mut is_main = true;
    // without func_sig
    for acir in generated_acirs.into_iter() {
        let circuit_artifact = convert_generated_acir_into_circuit_without_signature(
            acir,
            // TODO: get rid of these clones
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
        );
        program_artifact.add_circuit(circuit_artifact, is_main);
        is_main = false;
    }
    program_artifact.brillig_names = brillig_function_names;

    Ok(program_artifact)
}

/// Compiles the given FunctionBuilder into a CompiledProgram
/// its taken from noirc_driver::compile_no_check, but modified to accept ArtifactsAndWarnings
pub fn compile_from_artifacts(
    artifacts: ArtifactsAndWarnings,
    abi: Abi,
) -> Result<CompiledProgram, CompileError> {
    let SsaProgramArtifact { program, debug, warnings, names, brillig_names, .. } =
        create_program(artifacts)?;
    let file_map = BTreeMap::new();
    Ok(CompiledProgram {
        hash: 1, // const hash, doesn't matter in this case
        program,
        debug,
        abi,
        file_map,
        noir_version: NOIR_ARTIFACT_VERSION_STRING.to_string(),
        warnings,
        names,
        brillig_names,
    })
}

pub fn evaluator_options(options: &CompileOptions) -> SsaEvaluatorOptions {
    SsaEvaluatorOptions {
        ssa_logging: if !options.show_ssa_pass.is_empty() {
            SsaLogging::Contains(options.show_ssa_pass.clone())
        } else if options.show_ssa {
            SsaLogging::All
        } else {
            SsaLogging::None
        },
        print_codegen_timings: options.benchmark_codegen,
        expression_width: ExpressionWidth::default(),
        emit_ssa: { None },
        skip_underconstrained_check: options.skip_underconstrained_check,
        skip_brillig_constraints_check: options.skip_brillig_constraints_check,
        inliner_aggressiveness: options.inliner_aggressiveness,
        max_bytecode_increase_percent: options.max_bytecode_increase_percent,
        brillig_options: BrilligOptions::default(),
        enable_brillig_constraints_check_lookback: options
            .enable_brillig_constraints_check_lookback,
        skip_passes: options.skip_ssa_pass.clone(),
    }
}

pub(crate) fn compile_from_ssa(
    ssa: Ssa,
    options: &CompileOptions,
) -> Result<CompiledProgram, CompileError> {
    let artifacts = optimize_ssa_into_acir(ssa, evaluator_options(options))?;
    // ABI is not used during SSA execution, but its needed for compile function
    compile_from_artifacts(
        artifacts,
        Abi { parameters: vec![], return_type: None, error_types: BTreeMap::new() },
    )
}
