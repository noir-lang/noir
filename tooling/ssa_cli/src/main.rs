#![forbid(unsafe_code)]

use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};
mod cli;

fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::ACTIVE)
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_env_filter(EnvFilter::from_env("NOIR_LOG"))
        .init();

    if let Err(e) = cli::start_cli() {
        eprintln!("{e:#}");
        std::process::exit(1);
    }
}
