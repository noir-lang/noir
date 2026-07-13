use std::path::Path;

use color_eyre::eyre;
use noir_ast_fuzzer::DisplayAstAsNoir;
use noir_ast_fuzzer::compare::{
    CompareCompiled, CompareCompiledResult, CompareComptime, CompareInterpreted,
    CompareInterpretedResult, HasPrograms,
};
use noirc_abi::input_parser::Format;

use noirc_evaluator::ssa::{self, SsaEvaluatorOptions, SsaProgramArtifact};
use noirc_evaluator::ssa::{SsaPass, primary_passes};
use noirc_frontend::monomorphization::ast::Program;

pub mod targets;

fn bool_from_env(key: &str) -> bool {
    std::env::var(key).map(|s| matches!(s.as_str(), "1" | "true" | "yes")).unwrap_or_default()
}

/// Show all SSA passes during compilation.
fn show_ssa() -> bool {
    bool_from_env("NOIR_AST_FUZZER_SHOW_SSA")
}

pub fn default_ssa_options() -> SsaEvaluatorOptions {
    ssa::SsaEvaluatorOptions {
        ssa_logging: if show_ssa() { ssa::SsaLogging::All } else { ssa::SsaLogging::None },
        ..Default::default()
    }
}

/// Minimal `Nargo.toml` for a reproduction package emitted on failure.
const REPRO_NARGO_TOML: &str = "[package]\nname = \"fuzz_repro\"\ntype = \"bin\"\nauthors = []\n";

/// Target directory for reproduction projects, taken from `NOIR_AST_FUZZER_EMIT_PROJECT`.
///
/// When set, a failing target writes a runnable `nargo` package so the failure can be
/// replayed with `nargo execute` instead of rebuilding the project by hand.
fn emit_project_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("NOIR_AST_FUZZER_EMIT_PROJECT").map(Into::into)
}

/// Write a single `nargo` package: `Nargo.toml`, `src/main.nr`, and an optional `Prover.toml`.
///
/// The `main.nr` is the fuzzer's best-effort Noir rendering of the AST; for the compiled
/// targets it is not guaranteed to parse back, but it is a starting point that is otherwise
/// reconstructed by hand from the printed output.
fn write_nargo_package(dir: &Path, main_nr: &str, prover_toml: Option<&str>) {
    if let Err(e) = std::fs::create_dir_all(dir.join("src")) {
        eprintln!("failed to create project dir {}: {e}", dir.display());
        return;
    }
    let mut files = vec![("Nargo.toml", REPRO_NARGO_TOML), ("src/main.nr", main_nr)];
    if let Some(toml) = prover_toml {
        files.push(("Prover.toml", toml));
    }
    for (rel, contents) in files {
        let path = dir.join(rel);
        if let Err(e) = std::fs::write(&path, contents) {
            eprintln!("failed to write {}: {e}", path.display());
        }
    }
    eprintln!("--- Wrote nargo project to {}", dir.display());
}

/// Emit one `nargo` package per program. A single program is written directly under `dir`;
/// multiple programs are written under `dir/ast_1`, `dir/ast_2`, ..., all sharing the same
/// `Prover.toml` inputs.
fn emit_nargo_projects(dir: &Path, mains: &[String], prover_toml: Option<&str>) {
    match mains {
        [] => {}
        [main] => write_nargo_package(dir, main, prover_toml),
        many => {
            for (i, main) in many.iter().enumerate() {
                write_nargo_package(&dir.join(format!("ast_{}", i + 1)), main, prover_toml);
            }
        }
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
            if let Some(dir) = emit_project_dir() {
                // A compile-time failure has no ABI inputs, so emit `main.nr` only.
                write_nargo_package(&dir, &DisplayAstAsNoir(&program).to_string(), None);
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
        for (i, &ast) in asts.iter().enumerate() {
            if has_many {
                eprintln!("---\nAST {}:\n{}", i + 1, DisplayAstAsNoir(ast));
            } else {
                eprintln!("---\nAST:\n{}", DisplayAstAsNoir(ast));
            }
        }
        // Showing the inputs as TOML so we can easily create a Prover.toml file.
        let inputs_toml = Format::Toml.serialize(&inputs.input_map, &inputs.abi);
        eprintln!(
            "---\nInputs:\n{}",
            match &inputs_toml {
                Ok(toml) => toml.clone(),
                Err(e) => format!("failed to serialize inputs: {e}"),
            }
        );

        if let Some(dir) = emit_project_dir() {
            let mains =
                asts.iter().map(|&ast| DisplayAstAsNoir(ast).to_string()).collect::<Vec<_>>();
            emit_nargo_projects(&dir, &mains, inputs_toml.as_deref().ok());
        }

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
        display_program(&inputs.ssa2.artifact);

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
        eprintln!("---\nComptime source:\n{}", inputs.source);
        eprintln!("---\nAST:\n{}", DisplayAstAsNoir(&inputs.program));

        eprintln!("---\nCompile options:\n{:?}", inputs.ssa.options);
        eprintln!("---\nCompiled program:\n{}", inputs.ssa.artifact.program);

        if let Some(dir) = emit_project_dir() {
            // The comptime source is already valid Noir; the comptime call bakes in its inputs,
            // so there is no `Prover.toml` to emit.
            write_nargo_package(&dir, &inputs.source, None);
        }

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

        if let Some(dir) = emit_project_dir() {
            let toml = Format::Toml.serialize(&inputs.input_map, &inputs.abi).ok();
            write_nargo_package(
                &dir,
                &DisplayAstAsNoir(&inputs.program).to_string(),
                toml.as_deref(),
            );
        }

        // Returning it as-is, so we can see the error message at the bottom as well.
        Err(report)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::{emit_nargo_projects, write_nargo_package};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    /// A fresh, unique temp directory that does not yet exist on disk.
    fn temp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("noir_ast_fuzzer_emit_{}_{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn writes_single_package_with_inputs() {
        let dir = temp_dir();
        write_nargo_package(&dir, "fn main() {}", Some("x = \"1\"\n"));

        assert!(dir.join("Nargo.toml").is_file());
        assert_eq!(std::fs::read_to_string(dir.join("src/main.nr")).unwrap(), "fn main() {}");
        assert_eq!(std::fs::read_to_string(dir.join("Prover.toml")).unwrap(), "x = \"1\"\n");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn omits_prover_toml_when_no_inputs() {
        let dir = temp_dir();
        write_nargo_package(&dir, "fn main() {}", None);

        assert!(dir.join("src/main.nr").is_file());
        assert!(!dir.join("Prover.toml").exists());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn single_ast_is_written_directly_under_dir() {
        let dir = temp_dir();
        emit_nargo_projects(&dir, &["fn main() {}".to_string()], None);

        assert!(dir.join("src/main.nr").is_file());
        assert!(!dir.join("ast_1").exists());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn multiple_asts_are_written_under_numbered_subdirs() {
        let dir = temp_dir();
        let mains = ["fn main() { 1 }".to_string(), "fn main() { 2 }".to_string()];
        emit_nargo_projects(&dir, &mains, Some("y = \"2\"\n"));

        assert_eq!(
            std::fs::read_to_string(dir.join("ast_1/src/main.nr")).unwrap(),
            "fn main() { 1 }"
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("ast_2/src/main.nr")).unwrap(),
            "fn main() { 2 }"
        );
        // Shared inputs are written into every package.
        assert_eq!(std::fs::read_to_string(dir.join("ast_1/Prover.toml")).unwrap(), "y = \"2\"\n");
        assert_eq!(std::fs::read_to_string(dir.join("ast_2/Prover.toml")).unwrap(), "y = \"2\"\n");

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
