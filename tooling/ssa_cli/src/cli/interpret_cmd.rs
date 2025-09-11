use std::path::PathBuf;

use clap::Args;
use noir_artifact_cli::commands::parse_and_normalize_path;

/// Parse the input SSA and it arguments, run the SSA interpreter,
/// then write the return values to stdout.
#[derive(Debug, Clone, Args)]
pub(super) struct InterpretCommand {
    /// Path to the arguments to the SSA interpreter.
    ///
    /// Expected to be in TOML format, similar to `Prover.toml`.
    ///
    /// If empty, we assume the SSA has no arguments.
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub args_path: Option<PathBuf>,
}
