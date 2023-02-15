#![forbid(unsafe_code)]

use color_eyre::config::HookBuilder;
use nargo::cli::start_cli;

fn main() {
    // Register a panic hook to display more readable panic messages to end-users
    let (panic_hook, _) = HookBuilder::default()
        .display_env_section(false)
        .panic_section("This is a bug. Consider opening an issue at https://github.com/noir-lang/noir/issues/new?labels=bug&template=bug_report.md")
        .into_hooks();
    panic_hook.install();

    start_cli();
}
