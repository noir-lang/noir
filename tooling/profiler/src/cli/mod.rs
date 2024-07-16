use clap::{Parser, Subcommand};
use color_eyre::eyre;
use const_format::formatcp;

mod gates_flamegraph_cmd;
mod opcodes_flamegraph_cmd;

const PROFILER_VERSION: &str = env!("CARGO_PKG_VERSION");

static VERSION_STRING: &str = formatcp!("version = {}\n", PROFILER_VERSION,);

#[derive(Parser, Debug)]
#[command(name="Noir profiler", author, version=VERSION_STRING, about, long_about = None)]
struct ProfilerCli {
    #[command(subcommand)]
    command: ProfilerCommand,
}

#[non_exhaustive]
#[derive(Subcommand, Clone, Debug)]
enum ProfilerCommand {
    GatesFlamegraph(gates_flamegraph_cmd::GatesFlamegraphCommand),
    OpcodesFlamegraph(opcodes_flamegraph_cmd::OpcodesFlamegraphCommand),
}

pub(crate) fn start_cli() -> eyre::Result<()> {
    let ProfilerCli { command } = ProfilerCli::parse();

    match command {
        ProfilerCommand::GatesFlamegraph(args) => gates_flamegraph_cmd::run(args),
        ProfilerCommand::OpcodesFlamegraph(args) => opcodes_flamegraph_cmd::run(args),
    }
    .map_err(|err| eyre::eyre!("{}", err))?;

    Ok(())
}
