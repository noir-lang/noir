use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre::{self, Context, bail};
use noir_artifact_cli::commands::parse_and_normalize_path;
use noirc_driver::CompileOptions;
use noirc_evaluator::ssa::ssa_gen::Ssa;

/// Parse the input SSA, run a specific SSA pass on it, then write the output SSA.
#[derive(Debug, Clone, Args)]
pub(super) struct TransformCommand {
    /// Path to write the output SSA to.
    ///
    /// If empty, the SSA will be written to stdout.
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub output_path: Option<PathBuf>,

    /// Name of the SSA pass to apply.
    ///
    /// We use the first match in the default SSA pipeline that contains the phrase.
    #[clap(long, short = 'p')]
    pub ssa_pass: String,

    #[clap(flatten)]
    pub(super) compile_options: CompileOptions,
}

pub(super) fn run(args: TransformCommand, ssa: Ssa) -> eyre::Result<()> {
    let options = args.compile_options.as_ssa_options(PathBuf::default());
    let passes = super::ssa_passes(&options);

    let Some((msg, pass)) =
        passes.iter().find(|(msg, _)| msg.to_lowercase().contains(&args.ssa_pass.to_lowercase()))
    else {
        bail!("cannot find SSA pass: '{}'", args.ssa_pass);
    };

    let ssa = pass.run(ssa).wrap_err_with(|| format!("failed to run pass '{msg}'"))?;

    // Print it so that that it can be piped back to the CLI.
    let output = format!("// After {msg}:\n{ssa}");

    if let Some(path) = args.output_path {
        noir_artifact_cli::fs::artifact::write_to_file(output.as_bytes(), &path)
            .wrap_err_with(|| format!("failed to write SSA to {}", path.to_string_lossy()))?;
    } else {
        println!("{output}");
    }

    Ok(())
}
