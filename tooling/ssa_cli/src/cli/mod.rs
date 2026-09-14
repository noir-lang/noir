use std::io::{IsTerminal, Read};
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use color_eyre::eyre::{self, Context, bail};
use const_format::formatcp;
use noir_artifact_cli::commands::parse_and_normalize_path;
use noir_artifact_cli::fs::artifact::write_to_file;
use noirc_driver::CompileOptions;
use noirc_errors::{println_to_stderr, println_to_stdout};
use noirc_evaluator::ssa::{
    SsaEvaluatorOptions, SsaPass, primary_passes,
    ssa_gen::{Ssa, validate_ssa_or_err},
};

/// Which validation ruleset to run on the source SSA.
#[derive(ValueEnum, Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ValidationMode {
    /// Skip validation entirely. Use to test how invalid input behaves.
    Off,
    /// Run every rule. Appropriate for hand-written input SSA or fully optimized SSA.
    #[default]
    Full,
    /// Run only the rules that must hold at every point in the pipeline, assuming the SSA is the
    /// result of applying a pass to already-valid SSA. Use this to validate the output of
    /// `transform`, so a pass's provably-safe-but-syntactically-unguarded output isn't rejected.
    AfterPass,
}

mod interpret_cmd;
mod transform_cmd;
mod visualize_cmd;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
static VERSION_STRING: &str = formatcp!("version = {}\n", PKG_VERSION,);

#[derive(Parser, Debug)]
#[command(name="noir-ssa", author, version=VERSION_STRING, about, long_about = None)]
struct SsaCli {
    #[command(subcommand)]
    command: SsaCommand,

    #[command(flatten)]
    args: SsaArgs,
}

/// Common SSA command parameters.
#[derive(Args, Clone, Debug)]
struct SsaArgs {
    /// Path to the source SSA.
    ///
    /// If empty, the SSA will be read from stdin.
    #[clap(long, short, global = true, value_parser = parse_and_normalize_path)]
    source_path: Option<PathBuf>,

    /// Which validation ruleset to run on the source SSA.
    ///
    /// `off` skips validation (to test how invalid input behaves); `full` runs every rule;
    /// `after-pass` runs only the rules that hold between passes, which is
    /// what you want when validating the output of `transform`.
    #[clap(long, global = true, value_enum, default_value_t = ValidationMode::Full)]
    validation_mode: ValidationMode,
}

#[derive(Subcommand, Clone, Debug)]
enum SsaCommand {
    /// List the SSA passes we can apply.
    List,
    /// Parse and (optionally) validate the SSA.
    /// Prints the normalized SSA, with canonical ID assignment.
    Check,
    Interpret(interpret_cmd::InterpretCommand),
    Transform(transform_cmd::TransformCommand),
    Visualize(visualize_cmd::VisualizeCommand),
}

pub(crate) fn start_cli() -> eyre::Result<()> {
    let SsaCli { command, args } = SsaCli::parse();

    let ssa = || {
        let src = read_source(args.source_path)?;
        parse_ssa(&src, args.validation_mode)
    };

    match command {
        SsaCommand::List => {
            // This command doesn't actually use the common parameters, but we could potentially
            // read the source, and figure out which passes we can apply to it based on its state.
            let options = CompileOptions::default().as_ssa_options(Default::default());
            for (msg, _) in ssa_passes(&options) {
                println_to_stdout!("{msg}");
            }
        }
        SsaCommand::Check => {
            // Parsing normalizes the SSA, so we just need to print it.
            println_to_stdout!("{}", ssa()?);
        }
        SsaCommand::Interpret(cmd) => interpret_cmd::run(cmd, ssa()?)?,
        SsaCommand::Transform(cmd) => transform_cmd::run(cmd, ssa()?)?,
        SsaCommand::Visualize(cmd) => visualize_cmd::run(cmd, ssa()?)?,
    }

    Ok(())
}

/// Read the SSA from a file or stdin.
fn read_source(path: Option<PathBuf>) -> eyre::Result<String> {
    if let Some(path) = path {
        std::fs::read_to_string(&path)
            .wrap_err_with(|| format!("failed to read the SSA from {}", path.to_string_lossy()))
    } else {
        let mut src = String::new();
        let stdin = std::io::stdin();

        if stdin.is_terminal() {
            // If we are in terminal mode, we can type in the SSA, but that's unlikely
            // what we wanted to achieve, and I'm not sure how to even signal EOF.
            bail!("The CLI is in terminal mode. Expected to read the SSA from a pipe.")
        }

        let mut handle = stdin.lock();
        handle.read_to_string(&mut src)?;
        Ok(src)
    }
}

/// Write the output to a file or stdout.
fn write_output(output: &str, path: Option<PathBuf>) -> eyre::Result<()> {
    if let Some(path) = path {
        write_to_file(output.as_bytes(), &path)
            .wrap_err_with(|| format!("failed to write output to {}", path.to_string_lossy()))?;
    } else {
        println_to_stdout!("{output}");
    }
    Ok(())
}

/// Parse the SSA.
///
/// If parsing fails, print errors to `stderr` and return a failure.
///
/// A semantic error under `Full` or `AfterPass` causes a panic. `mode` selects whether to skip
/// validation, run the full ruleset, or run only the between-passes subset (see [`ValidationMode`]).
fn parse_ssa(src: &str, mode: ValidationMode) -> eyre::Result<Ssa> {
    // Running the full ruleset while parsing gives source-annotated error spans; otherwise parse
    // without validation and run the chosen ruleset explicitly.
    let result = if mode == ValidationMode::Full {
        Ssa::from_str(src)
    } else {
        Ssa::from_str_no_validation(src)
    };
    let ssa = match result {
        Ok(ssa) => ssa,
        Err(source_with_errors) => {
            println_to_stderr!("{source_with_errors:?}");
            bail!("Failed to parse the SSA.")
        }
    };

    if mode == ValidationMode::AfterPass {
        return validate_ssa_or_err(ssa, false)
            .wrap_err("SSA failed the between-passes validation");
    }

    Ok(ssa)
}

/// List of the SSA passes in the primary pipeline, enriched with their "step"
/// count so we can use unambiguous naming in filtering.
fn ssa_passes(options: &SsaEvaluatorOptions) -> Vec<(String, SsaPass<'_>)> {
    let passes = primary_passes(options).into_iter();
    let length = passes.len();
    passes
        .enumerate()
        .map(|(i, pass)| {
            let last_step = i == length - 1;
            let last_step_msg = if last_step { " (last step)" } else { "" };
            let msg = format!("{} (step {}){last_step_msg}", pass.msg(), i + 1);
            (msg, pass)
        })
        .collect()
}
