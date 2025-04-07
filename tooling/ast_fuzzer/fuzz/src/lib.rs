use color_eyre::eyre;
use noir_ast_fuzzer::compare::{CompareResult, CompareSsa};
use noirc_evaluator::ssa::{self, SsaEvaluatorOptions, SsaProgramArtifact};
use noirc_frontend::monomorphization::ast::Program;

fn should_print_ast() -> bool {
    std::env::var("NOIR_AST_FUZZER_DEBUG").map(|s| s == "1" || s == "true").unwrap_or_default()
}

/// Compile a [Program] into SSA or panic.
///
/// Prints the AST if `NOIR_AST_FUZZER_DEBUG` is set.
pub fn create_ssa_or_die(
    program: Program,
    options: &SsaEvaluatorOptions,
    msg: Option<&str>,
) -> SsaProgramArtifact {
    // Unfortunately we can't use `std::panic::catch_unwind`
    // and `std::panic::resume_unwind` to catch any panic
    // and print the AST, then resume the panic, because
    // `Program` has a `RefCell` in it, which is not unwind safe.
    if should_print_ast() {
        eprintln!("---\n{program}\n---");
        eprintln!("---\n{program:?}\n---");
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
        for (i, ast) in asts(&inputs).into_iter().enumerate() {
            eprintln!("AST {}:\n{}", i + 1, ast);
        }
        eprintln!("Inputs:\n{:?}", inputs.input_map);
        eprintln!("Program 1:\n{}", inputs.ssa1.program);
        eprintln!("Program 2:\n{}", inputs.ssa2.program);
    }

    res.map(|_| ())
}
