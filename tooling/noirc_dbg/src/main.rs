#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

//! The debugger for Noir language.

// #[allow(deprecated)]
// use barretenberg_blackbox_solver::BarretenbergSolver;

mod app;
mod compile;
mod dap_server;
mod error;
mod vm;

use app::{App, State};

use compile::compile;

fn main() {
    let mut app = App::initialize();
    while !matches!(app.state, State::Exit) {
        if let Err(e) = app.run() {
            eprintln!("{}", e);
        }
    }
}
