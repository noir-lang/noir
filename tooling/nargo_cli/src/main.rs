#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

//! Nargo is the package manager for Noir
//! This name was used because it sounds like `cargo` and
//! Noir Package Manager abbreviated is npm, which is already taken.

mod backends;
mod cli;
mod errors;

use std::env;

use color_eyre::config::HookBuilder;
use env_logger::{Builder, Env};
use tracing_appender::rolling;

const PANIC_MESSAGE: &str = "This is a bug. We may have already fixed this in newer versions of Nargo so try searching for similar issues at https://github.com/noir-lang/noir/issues/.\nIf there isn't an open issue for this bug, consider opening one at https://github.com/noir-lang/noir/issues/new?labels=bug&template=bug_report.yml";

fn main() {
    let env = Env::default().filter_or("NOIR_LOG", "error"); // Default to 'error' if NOIR_LOG is not set
    Builder::from_env(env).init();

    // Setup tracing
    if let Ok(log_dir) = env::var("NARGO_LOG_DIR") {
        let debug_file = rolling::daily(log_dir, "nargo-log");
        tracing_subscriber::fmt()
            .with_writer(debug_file)
            .with_ansi(false)
            .with_max_level(tracing::Level::TRACE)
            .init();
    }

    // Register a panic hook to display more readable panic messages to end-users
    let (panic_hook, _) =
        HookBuilder::default().display_env_section(false).panic_section(PANIC_MESSAGE).into_hooks();
    panic_hook.install();

    if let Err(report) = cli::start_cli() {
        eprintln!("{report}");
        std::process::exit(1);
    }
}
