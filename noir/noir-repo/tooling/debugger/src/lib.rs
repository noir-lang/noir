mod context;
mod dap;
pub mod errors;
mod foreign_calls;
mod repl;
mod source_code_printer;

use std::io::{Read, Write};

use ::dap::errors::ServerError;
use ::dap::server::Server;
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use nargo::artifacts::debug::DebugArtifact;

use nargo::NargoError;
use noirc_driver::CompiledProgram;

pub fn debug_circuit<B: BlackBoxFunctionSolver>(
    blackbox_solver: &B,
    circuit: &Circuit,
    debug_artifact: DebugArtifact,
    initial_witness: WitnessMap,
) -> Result<Option<WitnessMap>, NargoError> {
    repl::run(blackbox_solver, circuit, &debug_artifact, initial_witness)
}

pub fn run_dap_loop<R: Read, W: Write, B: BlackBoxFunctionSolver>(
    server: Server<R, W>,
    solver: &B,
    program: CompiledProgram,
    initial_witness: WitnessMap,
) -> Result<(), ServerError> {
    dap::run_session(server, solver, program, initial_witness)
}
