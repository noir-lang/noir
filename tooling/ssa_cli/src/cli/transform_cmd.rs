use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre::{self, Context, bail};
use noir_artifact_cli::commands::parse_and_normalize_path;
use noirc_driver::CompileOptions;
use noirc_errors::println_to_stderr;
use noirc_evaluator::ssa::{SsaLogging, SsaPass, ssa_gen::Ssa};

use crate::cli::write_output;

/// Parse the input SSA, run some SSA passes on it, then write the output SSA.
#[derive(Debug, Clone, Args)]
pub(super) struct TransformCommand {
    /// Path to write the output SSA to.
    ///
    /// If empty, the SSA will be written to stdout.
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub output_path: Option<PathBuf>,

    /// Name of the SSA pass(es) to apply.
    ///
    /// The names are used to look up the first matching pass in the default pipeline,
    /// and apply them in the order of appearance (potentially multiple times).
    ///
    /// If no pass is specified, it applies all passes in the default pipeline.
    #[clap(long, short = 'p')]
    pub ssa_pass: Vec<String>,

    #[clap(flatten)]
    pub(super) compile_options: CompileOptions,
}

pub(super) fn run(args: TransformCommand, mut ssa: Ssa) -> eyre::Result<()> {
    let options = args.compile_options.as_ssa_options(PathBuf::default());
    let passes = super::ssa_passes(&options);

    let mut msg = "Initial";

    if args.ssa_pass.is_empty() {
        for pass in &passes {
            (ssa, msg) = run_pass(ssa, pass, &options.ssa_logging)?;
        }
    } else {
        for name in args.ssa_pass {
            let Some(pass) =
                passes.iter().find(|(msg, _)| msg.to_lowercase().contains(&name.to_lowercase()))
            else {
                bail!(
                    "cannot find SSA pass (use the `vector` command to see available passes): '{}'",
                    name
                );
            };
            (ssa, msg) = run_pass(ssa, pass, &options.ssa_logging)?;
        }
    }

    // Print the final state so that that it can be piped back to the CLI.
    let output = format_ssa(&mut ssa, msg, true);

    write_output(&output, args.output_path)
}

/// Run an SSA pass, and optionally print to `stderr`, distinct from `stdout` where the final result goes.
fn run_pass<'a>(
    ssa: Ssa,
    (msg, pass): &'a (String, SsaPass<'_>),
    ssa_logging: &SsaLogging,
) -> eyre::Result<(Ssa, &'a String)> {
    let mut ssa = pass.run(ssa).wrap_err_with(|| format!("failed to run pass '{msg}'"))?;

    if ssa_logging.matches(msg) {
        println_to_stderr!("{}", format_ssa(&mut ssa, msg, false));
    }

    Ok((ssa, msg))
}

/// Render the SSA to a string.
fn format_ssa(ssa: &mut Ssa, msg: &str, parsable: bool) -> String {
    // Differentiate between log output and the final one by whether the "After" is commented out.
    let prefix = if parsable { "// " } else { "" };

    // Make sure variable IDs are consistent.
    ssa.normalize_ids();

    format!("{prefix}After {msg}:\n{ssa}")
}
