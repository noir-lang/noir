mod context;
mod dap;
pub mod errors;
mod foreign_calls;
mod repl;
mod source_code_printer;

use std::io::{Read, Write};

use ::dap::errors::ServerError;
use ::dap::server::Server;
use acvm::acir::circuit::brillig::BrilligBytecode;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};
use acvm::{BlackBoxFunctionSolver, FieldElement};

use nargo::artifacts::debug::DebugArtifact;

use nargo::NargoError;
use noirc_driver::CompiledProgram;

pub fn debug_circuit<B: BlackBoxFunctionSolver<FieldElement>>(
    blackbox_solver: &B,
    circuit: &Circuit<FieldElement>,
    debug_artifact: DebugArtifact,
    initial_witness: WitnessMap<FieldElement>,
    unconstrained_functions: &[BrilligBytecode<FieldElement>],
) -> Result<Option<WitnessMap<FieldElement>>, NargoError> {
    repl::run(blackbox_solver, circuit, &debug_artifact, initial_witness, unconstrained_functions)
}

pub fn run_dap_loop<R: Read, W: Write, B: BlackBoxFunctionSolver<FieldElement>>(
    server: Server<R, W>,
    solver: &B,
    program: CompiledProgram,
    initial_witness: WitnessMap<FieldElement>,
) -> Result<(), ServerError> {
    dap::run_session(server, solver, program, initial_witness)
}
