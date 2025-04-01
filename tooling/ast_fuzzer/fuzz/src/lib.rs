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
        eprint!("---\n{program}\n---");
        eprint!("---\n{program:?}\n---");
    }

    ssa::create_program(program, options).unwrap_or_else(|e| {
        panic!(
            "failed to compile program: {}{e}",
            msg.map(|s| format!("{s}: ")).unwrap_or_default()
        )
    })
}
