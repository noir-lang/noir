#![forbid(unsafe_code)]
#![warn(unreachable_pub)]

use clap::{command, Parser, Subcommand};
use color_eyre::eyre;
use const_format::formatcp;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

use noir_artifact_cli::commands::execute_cmd;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
static VERSION_STRING: &str = formatcp!("version = {}\n", PKG_VERSION,);

#[derive(Parser, Debug)]
#[command(name="noir-execute", author, version=VERSION_STRING, about, long_about = None)]
struct ExecutorCli {
    #[command(flatten)]
    command: execute_cmd::ExecuteCommand,
}

#[non_exhaustive]
#[derive(Subcommand, Clone, Debug)]
enum ArtifactCommand {
    Execute(execute_cmd::ExecuteCommand),
}

pub fn start_cli() -> eyre::Result<()> {
    let ExecutorCli { command } = ExecutorCli::parse();

    execute_cmd::run(command)?;

    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::ACTIVE)
        .with_ansi(true)
        .with_env_filter(EnvFilter::from_env("NOIR_LOG"))
        .init();

    if let Err(e) = start_cli() {
        eprintln!("{e:?}");
        std::process::exit(1);
    }
}
