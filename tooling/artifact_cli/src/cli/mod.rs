use std::path::PathBuf;

use clap::{command, Parser, Subcommand};
use color_eyre::eyre;
use const_format::formatcp;
use eyre::eyre;

mod execute_cmd;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
static VERSION_STRING: &str = formatcp!("version = {}\n", PKG_VERSION,);

#[derive(Parser, Debug)]
#[command(name="noir-artifact", author, version=VERSION_STRING, about, long_about = None)]
struct ArtifactCli {
    #[command(subcommand)]
    command: ArtifactCommand,
}

#[non_exhaustive]
#[derive(Subcommand, Clone, Debug)]
enum ArtifactCommand {
    Execute(execute_cmd::ExecuteCommand),
}

pub(crate) fn start_cli() -> eyre::Result<()> {
    let ArtifactCli { command } = ArtifactCli::parse();

    match command {
        ArtifactCommand::Execute(args) => execute_cmd::run(args),
    }
}

/// Parses a path and turns it into an absolute one by joining to the current directory,
/// then normalizes it.
fn parse_and_normalize_path(path: &str) -> eyre::Result<PathBuf> {
    use fm::NormalizePath;
    let mut path: PathBuf = path.parse().map_err(|e| eyre!("failed to parse path: {e}"))?;
    if !path.is_absolute() {
        path = std::env::current_dir().unwrap().join(path).normalize();
    }
    Ok(path)
}
