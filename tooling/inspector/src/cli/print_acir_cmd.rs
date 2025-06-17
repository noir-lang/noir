use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre;

#[derive(Debug, Clone, Args)]
pub(crate) struct PrintAcirCommand {
    /// The artifact to print
    artifact: PathBuf,

    /// Name of the function to print, if the artifact is a contract.
    #[clap(long)]
    contract_fn: Option<String>,
}

pub(crate) fn run(_args: PrintAcirCommand) -> eyre::Result<()> {
    eprintln!("Print ACIR command is not available in Sensei (requires ZK backend)");
    Err(eyre::eyre!("ACIR inspection is not available without ACVM backend"))
}
