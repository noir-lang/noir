#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

//! Nargo is the package manager for Noir
//! This name was used because it sounds like `cargo` and
//! Noir Package Manager abbreviated is npm, which is already taken.

mod cli;
mod errors;

use std::env;

use color_eyre::config::HookBuilder;

use tracing_appender::rolling;
use tracing_subscriber::{filter::LevelFilter, fmt::format::FmtSpan, EnvFilter};

const PANIC_MESSAGE: &str = "This is a bug. We may have already fixed this in newer versions of Nargo so try searching for similar issues at https://github.com/blocksense-network/noir/issues/.\nIf there isn't an open issue for this bug, consider opening one at https://github.com/blocksense-network/noir/issues/new";

fn main() {
    setup_tracing();

    // Register a panic hook to display more readable panic messages to end-users
    let (panic_hook, _) =
        HookBuilder::default().display_env_section(false).panic_section(PANIC_MESSAGE).into_hooks();
    panic_hook.install();

    if let Err(report) = cli::start_cli() {
        eprintln!("Error: {report}");
        std::process::exit(1);
    }
}

fn setup_tracing() {
    let subscriber = tracing_subscriber::fmt().with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE);

    if let Ok(log_dir) = env::var("NARGO_LOG_DIR") {
        let debug_file = rolling::daily(log_dir, "nargo-log");
        subscriber
            .with_writer(debug_file)
            .with_ansi(false)
            .with_env_filter(EnvFilter::from_env("NOIR_LOG"))
            .json()
            .init();
    } else {
        subscriber
            .with_ansi(true)
            .with_env_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::WARN.into())
                    .with_env_var(&"NOIR_LOG")
                    .from_env_lossy(),
            )
            .init();
    }
}
