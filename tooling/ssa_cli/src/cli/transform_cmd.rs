use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre::{self, Context, bail};
use noir_artifact_cli::commands::parse_and_normalize_path;
use noirc_driver::CompileOptions;
use noirc_evaluator::ssa::{SsaPass, ssa_gen::Ssa};

/// Parse the input SSA, run a specific SSA pass on it, then write the output SSA.
#[derive(Debug, Clone, Args)]
pub(super) struct TransformCommand {
    /// Path to write the output SSA to.
    ///
    /// If empty, the SSA will be written to stdout.
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub output_path: Option<PathBuf>,

    /// Name of the SSA pass(es) to apply.
    ///
    /// The name is used to look up the first matching pass in the default pipeline.
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
            (ssa, msg) = run_pass(ssa, pass)?;
        }
    } else {
        for name in args.ssa_pass {
            let Some(pass) =
                passes.iter().find(|(msg, _)| msg.to_lowercase().contains(&name.to_lowercase()))
            else {
                bail!(
                    "cannot find SSA pass (use the `list` command to see available passes): '{}'",
                    name
                );
            };
            (ssa, msg) = run_pass(ssa, pass)?;
        }
    }

    // Print the final state so that that it can be piped back to the CLI.
    let output = format_ssa(&ssa, msg);

    if let Some(path) = args.output_path {
        noir_artifact_cli::fs::artifact::write_to_file(output.as_bytes(), &path)
            .wrap_err_with(|| format!("failed to write SSA to {}", path.to_string_lossy()))?;
    } else {
        println!("{output}");
    }

    Ok(())
}

/// Run an SSA pass, and optionally print to `stderr`, distinct from `stdout` where the final result goes.
fn run_pass<'a>(
    ssa: Ssa,
    (msg, pass): &'a (String, SsaPass<'_>),
) -> eyre::Result<(Ssa, &'a String)> {
    let ssa = pass.run(ssa).wrap_err_with(|| format!("failed to run pass '{msg}'"))?;

    Ok((ssa, msg))
}

/// Format the SSA so that it can be printed and parsed back.
fn format_ssa(ssa: &Ssa, msg: &str) -> String {
    format!("// After {msg}:\n{ssa}")
}
