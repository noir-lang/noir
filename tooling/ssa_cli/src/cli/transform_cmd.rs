use std::path::PathBuf;

use clap::Args;
use noir_artifact_cli::commands::parse_and_normalize_path;

/// Parse the input SSA, run a specific SSA pass on it, then write the output SSA.
#[derive(Debug, Clone, Args)]
pub(super) struct TransformCommand {
    /// Path to write the output SSA to.
    ///
    /// If empty, the SSA will be written to stdout.
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub output_path: Option<PathBuf>,
}
