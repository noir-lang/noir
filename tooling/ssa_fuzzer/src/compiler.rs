use noir_ssa_executor::compiler::{
    compile_from_artifacts, evaluator_options, optimize_ssa_into_acir,
};
use noirc_abi::Abi;
use noirc_driver::{CompileError, CompileOptions, CompiledProgram};
use noirc_evaluator::{
    errors::RuntimeError,
    ssa::{ArtifactsAndWarnings, SsaEvaluatorOptions, function_builder::FunctionBuilder},
};
use std::collections::BTreeMap;

/// Optimizes the given FunctionBuilder into ACIR
/// its taken from noirc_evaluator::ssa::optimize_all, but modified to accept FunctionBuilder
/// and to catch panics... It cannot be caught with just catch_unwind.
/// This function will also run Ssa validation to make sure that the hand written Ssa has been well formed.
fn optimize_into_acir_and_validate(
    builder: FunctionBuilder,
    options: SsaEvaluatorOptions,
) -> Result<ArtifactsAndWarnings, RuntimeError> {
    let ssa = builder.finish();
    log::debug!("SSA: {:}", ssa);
    optimize_ssa_into_acir(ssa, options)
}

/// Compiles the given FunctionBuilder into a CompiledProgram
pub fn compile_from_builder(
    builder: FunctionBuilder,
    options: &CompileOptions,
) -> Result<CompiledProgram, CompileError> {
    let artifacts = optimize_into_acir_and_validate(builder, evaluator_options(options))?;
    compile_from_artifacts(
        artifacts,
        Abi { parameters: vec![], return_type: None, error_types: BTreeMap::new() },
    )
}
