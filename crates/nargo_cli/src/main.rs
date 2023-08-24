#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

//! Nargo is the package manager for Noir
//! This name was used because it sounds like `cargo` and
//! Noir Package Manager abbreviated is npm, which is already taken.

mod backends;
mod cli;
mod errors;

use color_eyre::{config::HookBuilder, eyre};

const PANIC_MESSAGE: &str = "This is a bug. We may have already fixed this in newer versions of Nargo so try searching for similar issues at https://github.com/noir-lang/noir/issues/.\nIf there isn't an open issue for this bug, consider opening one at https://github.com/noir-lang/noir/issues/new?labels=bug&template=bug_report.yml";

fn main() -> eyre::Result<()> {
    // Register a panic hook to display more readable panic messages to end-users
    let (panic_hook, _) =
        HookBuilder::default().display_env_section(false).panic_section(PANIC_MESSAGE).into_hooks();
    panic_hook.install();

    cli::start_cli()
}
