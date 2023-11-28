//! The solo debugger app for Noir language.
//! It generates and executes brillig bytecode.
//! The inputs are provided by prover file.

use noirc_dbg::{
    app::{App, State},
    dap_server::Dap,
};

fn main() {
    let mut app = App::initialize(Dap::new());
    while !matches!(app.state, State::Exit) {
        if let Err(e) = app.run() {
            eprintln!("{}", e);
        }
    }
}
