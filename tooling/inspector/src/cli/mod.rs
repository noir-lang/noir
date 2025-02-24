use clap::{command, Parser, Subcommand};
use color_eyre::eyre;
use const_format::formatcp;

mod info_cmd;
mod print_acir_cmd;

const INSPECTOR_VERSION: &str = env!("CARGO_PKG_VERSION");

static VERSION_STRING: &str = formatcp!("version = {}\n", INSPECTOR_VERSION,);

#[derive(Parser, Debug)]
#[command(name="Noir inspector", author, version=VERSION_STRING, about, long_about = None)]
struct InspectorCli {
    #[command(subcommand)]
    command: InspectorCommand,
}

#[non_exhaustive]
#[derive(Subcommand, Clone, Debug)]
enum InspectorCommand {
    Info(info_cmd::InfoCommand),
    PrintAcir(print_acir_cmd::PrintAcirCommand),
}

pub(crate) fn start_cli() -> eyre::Result<()> {
    let InspectorCli { command } = InspectorCli::parse();

    match command {
        InspectorCommand::Info(args) => info_cmd::run(args),
        InspectorCommand::PrintAcir(args) => print_acir_cmd::run(args),
    }
}
