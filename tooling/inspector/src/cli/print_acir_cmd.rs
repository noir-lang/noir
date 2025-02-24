use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre;
use noirc_artifacts::program::ProgramArtifact;

#[derive(Debug, Clone, Args)]
pub(crate) struct PrintAcirCommand {
    /// The artifact to print
    artifact: PathBuf,
}

pub(crate) fn run(args: PrintAcirCommand) -> eyre::Result<()> {
    let file = std::fs::File::open(args.artifact.clone())?;
    let artifact: ProgramArtifact = serde_json::from_reader(file)?;

    println!("Compiled ACIR for main:");
    println!("{}", artifact.bytecode);

    Ok(())
}
