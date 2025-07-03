use acvm::acir::circuit::ExpressionWidth;
use noirc_abi::Abi;
use noirc_driver::{CompileError, CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING};
use noirc_errors::call_stack::CallStack;
use noirc_evaluator::ssa::convert_generated_acir_into_circuit;
use noirc_evaluator::{
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
use std::collections::BTreeMap;
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

/// Creates a program artifact from the given FunctionBuilder
/// its taken from noirc_evaluator::ssa::create_program, but modified to accept FunctionBuilder
fn create_program(artifacts: ArtifactsAndWarnings) -> Result<SsaProgramArtifact, RuntimeError> {
    let ArtifactsAndWarnings(
        (generated_acirs, generated_brillig, brillig_function_names, error_types),
        ssa_level_warnings,
    ) = artifacts;

    let error_types = error_types
        .into_iter()
        .map(|(selector, hir_type)| (selector, ErrorType::Dynamic(hir_type)))
        .collect();

    let functions: Vec<SsaCircuitArtifact> = generated_acirs
        .into_iter()
        .map(|acir| {
            let dummy_arg_info = vec![(
                acir.input_witnesses.len() as u32,
                noirc_frontend::shared::Visibility::Private,
            )];
            convert_generated_acir_into_circuit(
                acir,
                &dummy_arg_info,
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
            )
        })
        .collect();

    let program_artifact = SsaProgramArtifact::new(
        functions,
        brillig_function_names,
        generated_brillig,
        error_types,
        ssa_level_warnings,
    );

    Ok(program_artifact)
}

/// Compiles the given FunctionBuilder into a CompiledProgram
/// its taken from noirc_driver::compile_no_check, but modified to accept ArtifactsAndWarnings
pub fn compile_from_artifacts(
    artifacts: ArtifactsAndWarnings,
) -> Result<CompiledProgram, CompileError> {
    let SsaProgramArtifact { program, debug, warnings, names, brillig_names, .. } =
        create_program(artifacts)?;
    let file_map = BTreeMap::new();
    Ok(CompiledProgram {
        hash: 1, // const hash, doesn't matter in this case
        program,
        debug,
        abi: Abi { parameters: vec![], return_type: None, error_types: BTreeMap::new() },
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
    compile_from_artifacts(artifacts)
}
