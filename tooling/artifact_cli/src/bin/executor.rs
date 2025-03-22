#![forbid(unsafe_code)]
#![warn(unreachable_pub)]

use clap::{Parser, Subcommand, command};
use color_eyre::eyre;
use const_format::formatcp;
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

use noir_artifact_cli::commands::execute_cmd;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
static VERSION_STRING: &str = formatcp!("version = {}\n", PKG_VERSION,);

#[derive(Parser, Debug)]
#[command(name="noir-executor", author, version=VERSION_STRING, about, long_about = None)]
struct ExecutorCli {
    #[command(subcommand)]
    command: ExecutorCommand,
}

#[non_exhaustive]
#[derive(Subcommand, Clone, Debug)]
enum ExecutorCommand {
    Execute(execute_cmd::ExecuteCommand),
    // TODO: Add other commands related to executing recordings from TypeScript; see https://github.com/AztecProtocol/aztec-packages/pull/12148
    // For example to decode an input map and render it as an ABI encoded prover file for integration tests.
}

pub fn start_cli() -> eyre::Result<()> {
    let ExecutorCli { command } = ExecutorCli::parse();

    match command {
        ExecutorCommand::Execute(cmd) => execute_cmd::run(cmd)?,
    }

    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::ACTIVE)
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_env_filter(EnvFilter::from_env("NOIR_LOG"))
        .init();

    if let Err(e) = start_cli() {
        eprintln!("{e:#}");
        std::process::exit(1);
    }
}
