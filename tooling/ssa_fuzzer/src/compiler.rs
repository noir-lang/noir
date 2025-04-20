use noir_ssa_executor::compiler::{
    compile_from_artifacts, evaluator_options, optimize_ssa_into_acir,
};
use noirc_driver::{CompileError, CompileOptions, CompiledProgram};
use noirc_evaluator::{
    errors::RuntimeError,
    ssa::{ArtifactsAndWarnings, SsaEvaluatorOptions, function_builder::FunctionBuilder},
};

/// Optimizes the given FunctionBuilder into ACIR
fn optimize_builder_into_acir(
    builder: FunctionBuilder,
    options: SsaEvaluatorOptions,
) -> Result<ArtifactsAndWarnings, RuntimeError> {
    let ssa = builder.finish();
    optimize_ssa_into_acir(ssa, options)
}

/// Compiles the given FunctionBuilder into a CompiledProgram
pub fn compile_from_builder(
    builder: FunctionBuilder,
    options: &CompileOptions,
) -> Result<CompiledProgram, CompileError> {
    let artifacts = optimize_builder_into_acir(builder, evaluator_options(options))?;
    compile_from_artifacts(artifacts)
}
