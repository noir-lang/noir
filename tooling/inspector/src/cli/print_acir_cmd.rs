use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre;
use noir_artifact_cli::Artifact;

#[derive(Debug, Clone, Args)]
pub(crate) struct PrintAcirCommand {
    /// The artifact to print
    artifact: PathBuf,

    /// Name of the function to print, if the artifact is a contract.
    #[clap(long)]
    contract_fn: Option<String>,
}

pub(crate) fn run(args: PrintAcirCommand) -> eyre::Result<()> {
    let artifact = Artifact::read_from_file(&args.artifact)?;

    match artifact {
        Artifact::Program(program) => {
            println!("Compiled ACIR for main:");
            println!("{}", program.bytecode);
        }
        Artifact::Contract(contract) => {
            println!("Compiled circuits for contract '{}':", contract.name);
            for function in contract
                .functions
                .into_iter()
                .filter(|f| args.contract_fn.as_ref().map(|n| *n == f.name).unwrap_or(true))
            {
                println!("Compiled ACIR for function '{}':", function.name);
                println!("{}", function.bytecode);
            }
        }
    }

    Ok(())
}
