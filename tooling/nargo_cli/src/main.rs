use std::env;

use color_eyre::config::HookBuilder;

use tracing_appender::rolling;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

use nargo_cli::{cli, PANIC_MESSAGE};

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
