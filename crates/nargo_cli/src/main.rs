#![forbid(unsafe_code)]

use color_eyre::{config::HookBuilder, eyre};
use nargo_cli::cli::start_cli;

const PANIC_MESSAGE: &str = "This is a bug. We may have already fixed this in newer versions of Nargo so try searching for similar issues at https://github.com/noir-lang/noir/issues/.\nIf there isn't an open issue for this bug, consider opening one at https://github.com/noir-lang/noir/issues/new?labels=bug&template=bug_report.yml";

fn main() -> eyre::Result<()> {
    // Register a panic hook to display more readable panic messages to end-users
    HookBuilder::default()
        .display_env_section(false)
        .display_location_section(false)
        .panic_section(PANIC_MESSAGE)
        .install()?;

    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "0");
    }

    start_cli()
}
