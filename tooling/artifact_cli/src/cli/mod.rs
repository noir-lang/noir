use clap::{command, Parser, Subcommand};
use color_eyre::eyre;
use const_format::formatcp;

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
