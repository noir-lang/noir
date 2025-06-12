use noir_ssa_executor::compiler::{
    compile_from_artifacts, evaluator_options, optimize_ssa_into_acir,
};
use noirc_abi::Abi;
use noirc_driver::{CompileError, CompileOptions, CompiledProgram};
use noirc_errors::call_stack::CallStack;
use noirc_evaluator::{
    errors::{InternalError, RuntimeError},
    ssa::{ArtifactsAndWarnings, SsaEvaluatorOptions, function_builder::FunctionBuilder},
};
use std::collections::BTreeMap;
use std::panic::AssertUnwindSafe;

/// Optimizes the given FunctionBuilder into ACIR
/// its taken from noirc_evaluator::ssa::optimize_all, but modified to accept FunctionBuilder
/// and to catch panics... It cannot be caught with just catch_unwind.
fn optimize_into_acir(
    builder: FunctionBuilder,
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
    let ssa = std::panic::catch_unwind(AssertUnwindSafe(|| builder.finish()));
    std::panic::set_hook(previous_hook);
    let error_msg = panic_message.lock().unwrap();

    match ssa {
        Ok(ssa) => optimize_ssa_into_acir(ssa, options),
        Err(_) => Err(RuntimeError::InternalError(InternalError::General {
            message: format!("Panic occurred: {}", error_msg),
            call_stack: CallStack::default(),
        })),
    }
}

/// Compiles the given FunctionBuilder into a CompiledProgram
pub fn compile_from_builder(
    builder: FunctionBuilder,
    options: &CompileOptions,
) -> Result<CompiledProgram, CompileError> {
    let artifacts = optimize_into_acir(builder, evaluator_options(options))?;
    compile_from_artifacts(
        artifacts,
        Abi { parameters: vec![], return_type: None, error_types: BTreeMap::new() },
    )
}
