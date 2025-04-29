use acir::circuit::ExpressionWidth;
use color_eyre::eyre;
use noir_ast_fuzzer::DisplayAstAsNoir;
use noir_ast_fuzzer::compare::{CompareResult, CompareSsa, HasPrograms};
use noirc_abi::input_parser::Format;
use noirc_evaluator::brillig::Brillig;
use noirc_evaluator::ssa::{SsaPass, primary_passes, secondary_passes};
use noirc_evaluator::{
    brillig::BrilligOptions,
    ssa::{self, SsaEvaluatorOptions, SsaProgramArtifact},
};
use noirc_frontend::monomorphization::ast::Program;

pub mod targets;

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
    create_ssa_with_passes_or_die(program, options, &primary_passes(options), secondary_passes, msg)
}

/// Compile a [Program] into SSA using the given primary and secondary passes, or panic.
///
/// Prints the AST if `NOIR_AST_FUZZER_SHOW_AST` is set.
pub fn create_ssa_with_passes_or_die<S>(
    program: Program,
    options: &SsaEvaluatorOptions,
    primary: &[SsaPass],
    secondary: S,
    msg: Option<&str>,
) -> SsaProgramArtifact
where
    S: for<'b> Fn(&'b Brillig) -> Vec<SsaPass<'b>>,
{
    // Unfortunately we can't use `std::panic::catch_unwind`
    // and `std::panic::resume_unwind` to catch any panic
    // and print the AST, then resume the panic, because
    // `Program` has a `RefCell` in it, which is not unwind safe.
    if show_ast() {
        eprintln!("---\n{}\n---", DisplayAstAsNoir(&program));
    }

    ssa::create_program_with_passes(program, options, primary, secondary).unwrap_or_else(|e| {
        panic!(
            "failed to compile program: {}{e}",
            msg.map(|s| format!("{s}: ")).unwrap_or_default()
        )
    })
}

/// Compare the execution result and print the inputs if the result is a failure.
pub fn compare_results<P>(inputs: &CompareSsa<P>, result: &CompareResult) -> eyre::Result<()>
where
    CompareSsa<P>: HasPrograms,
{
    let res = result.return_value_or_err();

    if let Err(report) = res {
        eprintln!("---\nComparison failed:");
        eprintln!("{report:#}");

        // Showing the AST as Noir so we can easily create integration tests.
        let asts = inputs.programs();
        let has_many = asts.len() > 1;
        for (i, ast) in asts.into_iter().enumerate() {
            if has_many {
                eprintln!("---\nAST {}:\n{}", i + 1, DisplayAstAsNoir(ast));
            } else {
                eprintln!("---\nAST:\n{}", DisplayAstAsNoir(ast));
            }
        }
        // Showing the inputs as TOML so we can easily create a Prover.toml file.
        eprintln!(
            "---\nInputs:\n{}",
            Format::Toml
                .serialize(&inputs.input_map, &inputs.abi)
                .unwrap_or_else(|e| format!("failed to serialize inputs: {e}"))
        );
        eprintln!("---\nOptions 1:\n{:?}", inputs.ssa1.options);
        eprintln!("---\nProgram 1:\n{}", inputs.ssa1.artifact.program);

        eprintln!("---\nOptions 2:\n{:?}", inputs.ssa2.options);
        eprintln!("---\nProgram 2:\n{}", inputs.ssa2.artifact.program);

        // Returning it as-is, so we can see the error message at the bottom as well.
        Err(report)
    } else {
        Ok(())
    }
}
