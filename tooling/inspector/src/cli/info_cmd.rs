use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre;

#[derive(Debug, Clone, Args)]
pub(crate) struct InfoCommand {
    /// The artifact to inspect
    artifact: PathBuf,

    /// Output a JSON formatted report. Changes to this format are not currently considered breaking.
    #[clap(long, hide = true)]
    json: bool,

    /// Name of the function to print, if the artifact is a contract.
    #[clap(long)]
    contract_fn: Option<String>,
}

pub(crate) fn run(_args: InfoCommand) -> eyre::Result<()> {
    eprintln!("Info command is not available in Sensei (requires ZK backend)");
    Err(eyre::eyre!("Artifact inspection is not available without ACVM backend"))
}
