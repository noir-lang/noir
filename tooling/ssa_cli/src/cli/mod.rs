use std::io::{IsTerminal, Read};
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, command};
use color_eyre::eyre::{self, Context, bail};
use const_format::formatcp;
use noir_artifact_cli::commands::parse_and_normalize_path;
use noir_artifact_cli::fs::artifact::write_to_file;
use noirc_driver::CompileOptions;
use noirc_errors::{println_to_stderr, println_to_stdout};
use noirc_evaluator::ssa::{SsaEvaluatorOptions, SsaPass, primary_passes, ssa_gen::Ssa};

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

    /// Turn off validation of the source SSA.
    ///
    /// This can be used to test how invalid input behaves.
    #[clap(long, global = true, default_value_t = false)]
    no_validate: bool,
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

    let ssa = || read_source(args.source_path).and_then(|src| parse_ssa(&src, !args.no_validate));

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
/// If validation is enabled, any semantic error causes a panic.
fn parse_ssa(src: &str, validate: bool) -> eyre::Result<Ssa> {
    let result = if validate { Ssa::from_str(src) } else { Ssa::from_str_no_validation(src) };
    match result {
        Ok(ssa) => Ok(ssa),
        Err(source_with_errors) => {
            println_to_stderr!("{source_with_errors:?}");
            bail!("Failed to parse the SSA.")
        }
    }
}

/// List of the SSA passes in the primary pipeline, enriched with their "step"
/// count so we can use unambiguous naming in filtering.
fn ssa_passes(options: &SsaEvaluatorOptions) -> Vec<(String, SsaPass<'_>)> {
    primary_passes(options)
        .into_iter()
        .enumerate()
        .map(|(i, pass)| {
            let msg = format!("{} (step {})", pass.msg(), i + 1);
            (msg, pass)
        })
        .collect()
}
