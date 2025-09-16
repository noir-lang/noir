use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre;
use noir_artifact_cli::commands::parse_and_normalize_path;
use noirc_evaluator::ssa::ssa_gen::Ssa;

use crate::cli::write_output;

/// Parse the SSA and render the CFG for visualization with Mermaid.
#[derive(Debug, Clone, Args)]
pub(super) struct VisualizeCommand {
    /// Path to write the output to.
    ///
    /// If empty, the output will be written to stdout.
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub output_path: Option<PathBuf>,

    /// Surround the output with syntax for direct embedding in a Markdown file.
    ///
    /// Useful for dumping the output into a file that can be previewed in VS Code for example.
    #[clap(long, short = 'p')]
    pub markdown: bool,
}

pub(super) fn run(args: VisualizeCommand, ssa: Ssa) -> eyre::Result<()> {
    // Print the final state so that that it can be piped back to the CLI.
    let output: String = todo!();

    write_output(&output, args.output_path)
}
