#![forbid(unsafe_code)]
#![warn(unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

//! Nargo is the package manager for Noir
//! This name was used because it sounds like `cargo` and
//! Noir Package Manager abbreviated is npm, which is already taken.

mod backends;
mod cli;
mod errors;

fn main() {
    panic::set_hook();

    if let Err(report) = cli::start_cli() {
        eprintln!("{report}");
        std::process::exit(1);
    }
}
