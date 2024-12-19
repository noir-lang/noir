use clap::{Parser, Subcommand};
use color_eyre::eyre;
use const_format::formatcp;

mod execute_cmd;
mod fs;

const ACVM_VERSION: &str = env!("CARGO_PKG_VERSION");

static VERSION_STRING: &str = formatcp!("version = {}\n", ACVM_VERSION,);

#[derive(Parser, Debug)]
#[command(name="acvm", author, version=VERSION_STRING, about, long_about = None)]
struct ACVMCli {
    #[command(subcommand)]
    command: ACVMCommand,
}

#[non_exhaustive]
#[derive(Subcommand, Clone, Debug)]
enum ACVMCommand {
    Execute(execute_cmd::ExecuteCommand),
}

pub(crate) fn start_cli() -> eyre::Result<()> {
    let ACVMCli { command } = ACVMCli::parse();

    match command {
        ACVMCommand::Execute(args) => execute_cmd::run(args),
    }?;

    Ok(())
}
