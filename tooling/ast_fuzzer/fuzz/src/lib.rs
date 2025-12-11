use color_eyre::eyre;
use noir_ast_fuzzer::DisplayAstAsNoir;
use noir_ast_fuzzer::compare::{
    CompareCompiled, CompareCompiledResult, CompareComptime, CompareInterpreted,
    CompareInterpretedResult, HasPrograms,
};
use noirc_abi::input_parser::Format;
use noirc_evaluator::ssa::opt::{CONSTANT_FOLDING_MAX_ITER, INLINING_MAX_INSTRUCTIONS};
use noirc_evaluator::ssa::{SsaPass, primary_passes};
use noirc_evaluator::{
    brillig::BrilligOptions,
    ssa::{self, OptimizationLevel, SsaEvaluatorOptions, SsaProgramArtifact},
};
use noirc_frontend::monomorphization::ast::Program;

pub mod targets;

fn bool_from_env(key: &str) -> bool {
    std::env::var(key).map(|s| s == "1" || s == "true").unwrap_or_default()
}

/// Show all SSA passes during compilation.
fn show_ssa() -> bool {
    bool_from_env("NOIR_AST_FUZZER_SHOW_SSA")
}

pub fn default_ssa_options() -> SsaEvaluatorOptions {
    ssa::SsaEvaluatorOptions {
        ssa_logging: if show_ssa() { ssa::SsaLogging::All } else { ssa::SsaLogging::None },
        optimization_level: OptimizationLevel::All,
        brillig_options: BrilligOptions::default(),
        print_codegen_timings: false,
        emit_ssa: None,
        skip_underconstrained_check: true,
        skip_brillig_constraints_check: true,
        enable_brillig_constraints_check_lookback: false,
        inliner_aggressiveness: 0,
        constant_folding_max_iter: CONSTANT_FOLDING_MAX_ITER,
        small_function_max_instruction: INLINING_MAX_INSTRUCTIONS,
        max_bytecode_increase_percent: None,
        skip_passes: Default::default(),
    }
}

/// Compile a monomorphized [Program] into circuit or panic.
pub fn compile_into_circuit_or_die(
    program: Program,
    options: &SsaEvaluatorOptions,
    msg: Option<&str>,
) -> SsaProgramArtifact {
    compile_into_circuit_with_ssa_passes_or_die(program, options, &primary_passes(options), msg)
}

/// Compile a monomorphized [Program] into circuit using the given SSA passes or panic.
///
/// If there is a seed in the environment, then it prints the AST when an error is encountered.
pub fn compile_into_circuit_with_ssa_passes_or_die(
    program: Program,
    options: &SsaEvaluatorOptions,
    primary: &[SsaPass],
    msg: Option<&str>,
) -> SsaProgramArtifact {
    // If we are using a seed, we are probably trying to reproduce some failure;
    // in that case let's clone the program for printing if it panicked here,
    // otherwise try to keep things faster by not cloning.
    let for_print = std::env::var("NOIR_AST_FUZZER_SEED").is_ok().then(|| program.clone());

    // We expect the programs generated should compile, but sometimes SSA passes panic, or return an error.
    // Turn everything into a panic, catch it, print the AST, then resume panicking.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ssa::create_program_with_passes(program.clone(), options, primary, None).unwrap_or_else(
            |e| {
                panic!(
                    "failed to compile program: {}{e}",
                    msg.map(|s| format!("{s}: ")).unwrap_or_default()
                )
            },
        )
    }));

    match result {
        Ok(ssa) => ssa,
        Err(payload) => {
            if let Some(program) = for_print {
                eprintln!("--- Failing AST:\n{}\n---", DisplayAstAsNoir(&program));
            }
            std::panic::resume_unwind(payload);
        }
    }
}

/// Compare the execution result and print the inputs if the result is a failure.
pub fn compare_results_compiled<P>(
    inputs: &CompareCompiled<P>,
    result: &CompareCompiledResult,
) -> eyre::Result<()>
where
    CompareCompiled<P>: HasPrograms,
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

        // Display a Program without the Brillig opcodes, which are unreadable.
        fn display_program(artifact: &SsaProgramArtifact) {
            for (func_index, function) in artifact.program.functions.iter().enumerate() {
                eprintln!("func {func_index}");
                eprintln!("{function}");
            }
            for (func_index, function) in
                artifact.program.unconstrained_functions.iter().enumerate()
            {
                eprintln!("unconstrained func {func_index}");
                eprintln!("opcode count: {}", function.bytecode.len());
            }
        }

        eprintln!("---\nOptions 1:\n{:?}", inputs.ssa1.options);
        eprintln!("---\nProgram 1:");
        display_program(&inputs.ssa1.artifact);

        eprintln!("---\nOptions 2:\n{:?}", inputs.ssa2.options);
        eprintln!("---\nProgram 2:");
        display_program(&inputs.ssa1.artifact);

        // Returning it as-is, so we can see the error message at the bottom as well.
        Err(report)
    } else {
        Ok(())
    }
}

/// Compare the execution result for comptime fuzzing and print the inputs if the result is a failure.
pub fn compare_results_comptime(
    inputs: &CompareComptime,
    result: &CompareCompiledResult,
) -> eyre::Result<()> {
    let res = result.return_value_or_err();

    if let Err(report) = res {
        eprintln!("---\nComparison failed:");
        eprintln!("{report:#}");

        // Showing the AST as Noir so we can easily create integration tests.
        eprintln!("---\nComptime source:\n{}", &inputs.source);
        eprintln!("---\nAST:\n{}", DisplayAstAsNoir(&inputs.program));

        eprintln!("---\nCompile options:\n{:?}", inputs.ssa.options);
        eprintln!("---\nCompiled program:\n{}", inputs.ssa.artifact.program);

        // Returning it as-is, so we can see the error message at the bottom as well.
        Err(report)
    } else {
        Ok(())
    }
}

/// Compare the execution result and print the inputs if the result is a failure.
pub fn compare_results_interpreted(
    inputs: &CompareInterpreted,
    result: &CompareInterpretedResult,
) -> eyre::Result<()> {
    let res = result.return_value_or_err();

    if let Err(report) = res {
        eprintln!("---\nComparison failed:");
        eprintln!("{report:#}");

        // Showing the AST as Noir so we can easily create integration tests.
        eprintln!("---\nAST:\n{}", DisplayAstAsNoir(&inputs.program));

        // Showing the inputs as TOML so we can easily create a Prover.toml file.
        eprintln!(
            "---\nABI Inputs:\n{}",
            Format::Toml
                .serialize(&inputs.input_map, &inputs.abi)
                .unwrap_or_else(|e| format!("failed to serialize inputs: {e}"))
        );

        // Show the SSA inputs as well so we can see any discrepancy in encoding.
        eprintln!(
            "---\nSSA Inputs:\n{}",
            inputs
                .ssa_args
                .iter()
                .enumerate()
                .map(|(i, v)| format!("{i}: {v}"))
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Common options for the SSA passes.
        eprintln!("---\nOptions:\n{:?}", inputs.options);

        eprintln!(
            "---\nSSA 1 after step {} ({}):\n{}",
            inputs.ssa1.step,
            inputs.ssa1.msg,
            inputs.ssa1.ssa.print_without_locations()
        );
        eprintln!(
            "---\nSSA 2 after step {} ({}):\n{}",
            inputs.ssa2.step,
            inputs.ssa2.msg,
            inputs.ssa2.ssa.print_without_locations()
        );

        // Returning it as-is, so we can see the error message at the bottom as well.
        Err(report)
    } else {
        Ok(())
    }
}
