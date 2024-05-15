mod context;
mod dap;
pub mod errors;
mod foreign_calls;
mod repl;
mod source_code_printer;

use std::io::{Read, Write};

use ::dap::errors::ServerError;
use ::dap::server::Server;
use acvm::acir::native_types::{WitnessMap, WitnessStack};
use acvm::BlackBoxFunctionSolver;

use nargo::NargoError;
use noirc_driver::CompiledProgram;

pub fn run_repl_session<B: BlackBoxFunctionSolver>(
    solver: &B,
    program: CompiledProgram,
    initial_witness: WitnessMap,
) -> Result<Option<WitnessStack>, NargoError> {
    repl::run(solver, program, initial_witness)
}

pub fn run_dap_loop<R: Read, W: Write, B: BlackBoxFunctionSolver>(
    server: Server<R, W>,
    solver: &B,
    program: CompiledProgram,
    initial_witness: WitnessMap,
) -> Result<(), ServerError> {
    dap::run_session(server, solver, program, initial_witness)
}
