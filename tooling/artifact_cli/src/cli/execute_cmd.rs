use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre::{self, Context};

use crate::fs::Artifact;

#[derive(Debug, Clone, Args)]
pub(crate) struct ExecuteCommand {
    /// Path to the JSON build artifact (either a program or a contract.)
    #[clap(long, short)]
    artifact: PathBuf,
}

pub(crate) fn run(args: ExecuteCommand) -> eyre::Result<()> {
    let artifact = Artifact::read_from_file(&args.artifact).context("failed to parse artifact")?;
    match artifact {
        Artifact::Program(program) => println!(
            "PROGRAM:\nnames: {:?}\nbrillig names:{:?}",
            program.names, program.brillig_names
        ),
        Artifact::Contract(contract) => println!(
            "CONTRACT: {}\nfunctions: {:?}",
            contract.name,
            contract.functions.iter().map(|f| f.name.clone()).collect::<Vec<_>>()
        ),
    }
    Ok(())
}
