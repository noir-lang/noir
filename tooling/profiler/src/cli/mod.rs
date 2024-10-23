use clap::{Parser, Subcommand};
use color_eyre::eyre;
use const_format::formatcp;

mod execution_flamegraph_cmd;
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
    Gates(gates_flamegraph_cmd::GatesFlamegraphCommand),
    Opcodes(opcodes_flamegraph_cmd::OpcodesFlamegraphCommand),
    ExecutionOpcodes(execution_flamegraph_cmd::ExecutionFlamegraphCommand),
}

pub(crate) fn start_cli() -> eyre::Result<()> {
    let ProfilerCli { command } = ProfilerCli::parse();

    match command {
        ProfilerCommand::Gates(args) => gates_flamegraph_cmd::run(args),
        ProfilerCommand::Opcodes(args) => opcodes_flamegraph_cmd::run(args),
        ProfilerCommand::ExecutionOpcodes(args) => execution_flamegraph_cmd::run(args),
    }
}
