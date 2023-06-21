#![forbid(unsafe_code)]


use color_eyre::{config::HookBuilder, eyre, Report};
use nargo_cli::{cli::start_cli, errors};


fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    // Register a panic hook to display more readable panic messages to end-users
    let (panic_hook, _) = HookBuilder::default()
        .display_env_section(false)
        .panic_section("This is a bug. We may have already fixed this in newer versions of Nargo so try searching for similar issues at https://github.com/noir-lang/noir/issues/.\nIf there isn't an open issue for this bug, consider opening one at https://github.com/noir-lang/noir/issues/new?labels=bug&template=bug_report.yml")
        .into_hooks();
    panic_hook.install();

    match start_cli() {
        Ok(exit_code) => {
            if exit_code > 0 {
                Err(Report::msg(format!("Backend returned error code {}", exit_code)))
            } else {
                // Ok(())
                Err(Report::msg(format!("Backend returned error code {}", exit_code)))
            }
        },
        Err(e) => Err(e),
    }
}
