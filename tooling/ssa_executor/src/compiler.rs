use noirc_abi::Abi;
use noirc_driver::{CompileError, CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING};
use noirc_errors::call_stack::CallStack;
use noirc_evaluator::{
    errors::{InternalError, RuntimeError},
    ssa::{
        ArtifactsAndWarnings, SsaBuilder, SsaEvaluatorOptions, SsaProgramArtifact,
        combine_artifacts, optimize_ssa_builder_into_acir, primary_passes, secondary_passes,
        ssa_gen::{Ssa, validate_ssa},
    },
};
use noirc_frontend::shared::Visibility;
use std::panic::AssertUnwindSafe;
use std::{collections::BTreeMap, path::PathBuf};

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
            format!("Panic: {s}")
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            format!("Panic: {s}")
        } else {
            format!("Unknown panic: {panic_info:?}")
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
                message: format!("Panic occurred: {error_msg}"),
                call_stack: CallStack::default(),
            }))
        }
    }
}

/// Compiles the given FunctionBuilder into a CompiledProgram
/// its taken from noirc_driver::compile_no_check, but modified to accept ArtifactsAndWarnings
pub fn compile_from_artifacts(artifacts: ArtifactsAndWarnings) -> CompiledProgram {
    let dummy_arg_info: Vec<Vec<(u32, Visibility)>> = artifacts
        .0
        .0
        .iter()
        .map(|acir| vec![(acir.input_witnesses.len() as u32, Visibility::Private)])
        .collect();

    let SsaProgramArtifact { program, debug, warnings, names, brillig_names, .. } =
        combine_artifacts(
            artifacts,
            &dummy_arg_info,
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
        );
    let file_map = BTreeMap::new();
    CompiledProgram {
        hash: 1, // const hash, doesn't matter in this case
        program,
        debug,
        abi: Abi { parameters: vec![], return_type: None, error_types: BTreeMap::new() },
        file_map,
        noir_version: NOIR_ARTIFACT_VERSION_STRING.to_string(),
        warnings,
        names,
        brillig_names,
    }
}

pub fn compile_from_ssa(
    ssa: Ssa,
    options: &CompileOptions,
) -> Result<CompiledProgram, CompileError> {
    let artifacts = optimize_ssa_into_acir(ssa, options.as_ssa_options(PathBuf::new()))?;
    Ok(compile_from_artifacts(artifacts))
}
