#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

//! Nargo is the package manager for Noir
//! This name was used because it sounds like `cargo` and
//! Noir Package Manager abbreviated is npm, which is already taken.

use nargo_cli::cli;

use std::env;

use color_eyre::config::HookBuilder;

use tracing_appender::rolling;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

// TODO: by making the cli command modules their own lib, cargo now
// 1. only considers these to be used by the lib
// 2. does not currently support lib-only dependencies:
//    https://github.com/rust-lang/cargo/issues/1982
use acvm as _;
use async_lsp as _;
use bn254_blackbox_solver as _;
use clap as _;
use const_format as _;
use dap as _;
use fm as _;
use iter_extended as _;
use nargo as _;
use nargo_fmt as _;
use nargo_toml as _;
use noir_debugger as _;
use noir_fuzzer as _;
use noir_lsp as _;
use noirc_abi as _;
use noirc_artifacts as _;
use noirc_driver as _;
use noirc_errors as _;
use noirc_frontend as _;
use notify as _;
use notify_debouncer_full as _;
use prettytable as _;
use proptest as _;
use rayon as _;
use serde as _;
use serde_json as _;
use similar_asserts as _;
use termcolor as _;
use termion as _;
use thiserror as _;
use tokio as _;
use toml as _;
use tower as _;

const PANIC_MESSAGE: &str = "This is a bug. We may have already fixed this in newer versions of Nargo so try searching for similar issues at https://github.com/noir-lang/noir/issues/.\nIf there isn't an open issue for this bug, consider opening one at https://github.com/noir-lang/noir/issues/new?labels=bug&template=bug_report.yml";

fn main() {
    // Setup tracing
    if let Ok(log_dir) = env::var("NARGO_LOG_DIR") {
        let debug_file = rolling::daily(log_dir, "nargo-log");
        tracing_subscriber::fmt()
            .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
            .with_writer(debug_file)
            .with_ansi(false)
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
            .with_ansi(true)
            .with_env_filter(EnvFilter::from_env("NOIR_LOG"))
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
