use acir::circuit::ExpressionWidth;
use color_eyre::eyre;
use noir_ast_fuzzer::compare::{CompareResult, CompareSsa};
use noirc_evaluator::{
    brillig::BrilligOptions,
    ssa::{self, SsaEvaluatorOptions, SsaProgramArtifact},
};
use noirc_frontend::monomorphization::ast::Program;

// TODO(#7876): Allow specifying options on the command line.
fn show_ast() -> bool {
    std::env::var("NOIR_AST_FUZZER_SHOW_AST").map(|s| s == "1" || s == "true").unwrap_or_default()
}

fn show_ssa() -> bool {
    std::env::var("NOIR_AST_FUZZER_SHOW_SSA").map(|s| s == "1" || s == "true").unwrap_or_default()
}

pub fn default_ssa_options() -> SsaEvaluatorOptions {
    ssa::SsaEvaluatorOptions {
        ssa_logging: if show_ssa() { ssa::SsaLogging::All } else { ssa::SsaLogging::None },
        brillig_options: BrilligOptions::default(),
        print_codegen_timings: false,
        expression_width: ExpressionWidth::default(),
        emit_ssa: None,
        skip_underconstrained_check: true,
        skip_brillig_constraints_check: true,
        enable_brillig_constraints_check_lookback: false,
        inliner_aggressiveness: 0,
        max_bytecode_increase_percent: None,
    }
}

/// Compile a [Program] into SSA or panic.
///
/// Prints the AST if `NOIR_AST_FUZZER_SHOW_AST` is set.
pub fn create_ssa_or_die(
    program: Program,
    options: &SsaEvaluatorOptions,
    msg: Option<&str>,
) -> SsaProgramArtifact {
    // Unfortunately we can't use `std::panic::catch_unwind`
    // and `std::panic::resume_unwind` to catch any panic
    // and print the AST, then resume the panic, because
    // `Program` has a `RefCell` in it, which is not unwind safe.
    if show_ast() {
        eprintln!("---\n{program}\n---");
    }

    ssa::create_program(program, options).unwrap_or_else(|e| {
        panic!(
            "failed to compile program: {}{e}",
            msg.map(|s| format!("{s}: ")).unwrap_or_default()
        )
    })
}

/// Compare the execution result and print the inputs if the result is a failure.
pub fn compare_results<'a, P, F, I>(
    inputs: &'a CompareSsa<P>,
    result: &CompareResult,
    asts: F,
) -> eyre::Result<()>
where
    F: Fn(&'a CompareSsa<P>) -> I,
    I: IntoIterator<Item = &'a Program>,
{
    let res = result.return_value_or_err();

    if res.is_err() {
        for (i, ast) in asts(inputs).into_iter().enumerate() {
            eprintln!("AST {}:\n{}", i + 1, ast);
        }
        eprintln!("Inputs:\n{:?}", inputs.input_map);
        eprintln!("Program 1:\n{}", inputs.ssa1.program);
        eprintln!("Program 2:\n{}", inputs.ssa2.program);
    }

    res.map(|_| ())
}
