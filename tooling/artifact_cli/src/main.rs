#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
//#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

mod cli;

fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::ACTIVE)
        .with_ansi(true)
        .with_env_filter(EnvFilter::from_env("NOIR_LOG"))
        .init();

    if let Err(e) = cli::start_cli() {
        eprintln!("{e:?}");
        std::process::exit(1);
    }
}
