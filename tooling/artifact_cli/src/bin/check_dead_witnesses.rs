#![forbid(unsafe_code)]

use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

use noir_artifact_cli::commands::check_dead_witnesses_cmd::{self, CheckDeadWitnessesCommand};

/// Check for dead witnesses in a compiled program artifact.
#[derive(Parser, Debug)]
#[command(name = "noir-check-dead-witnesses")]
struct Cli {
    #[command(flatten)]
    args: CheckDeadWitnessesCommand,
}

fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::ACTIVE)
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_env_filter(EnvFilter::from_env("NOIR_LOG"))
        .init();

    let cli = Cli::parse();

    if let Err(e) = check_dead_witnesses_cmd::run(cli.args) {
        eprintln!("{e:#}");
        std::process::exit(1);
    }
}
