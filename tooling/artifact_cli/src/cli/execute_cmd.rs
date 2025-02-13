use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre::{self, Context};

use noir_artifact_cli::fs::Artifact;

#[derive(Debug, Clone, Args)]
pub(crate) struct ExecuteCommand {
    /// Path to the JSON build artifact (either a program or a contract).
    #[clap(long, short)]
    artifact: PathBuf,

    /// Name of the function to execute, if the artifact is a contract.
    #[clap(long)]
    contract_fn: Option<String>,

    /// Path to the Prover.toml file which contains the inputs and the
    /// optional return value in ABI format.
    #[clap(long, short)]
    prover_file: PathBuf,

    /// Part to the Oracle.toml file which contains the Oracle transcript,
    /// which is a list of responses captured during an earlier execution,
    /// which can replayed via mocks.
    ///
    /// Note that a transcript might be invalid if the inputs change and
    /// the circuit takes a different path during execution.
    #[clap(long, conflicts_with = "oracle_resolver")]
    oracle_file: Option<String>,

    /// JSON RPC url to solve oracle calls.
    ///
    /// This is to facilitate new executions, as opposed to replays.
    #[clap(long, conflicts_with = "oracle_file")]
    oracle_resolver: Option<String>,
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
