use acvm::pwg::{ACVMStatus, ErrorLocation, OpcodeResolutionError, ACVM};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use crate::errors::ExecutionError;
use crate::NargoError;

use super::foreign_calls::ForeignCall;

pub fn execute_circuit<B: BlackBoxFunctionSolver>(
    blackbox_solver: &B,
    circuit: Circuit,
    initial_witness: WitnessMap,
    show_output: bool,
) -> Result<WitnessMap, NargoError> {
    let mut acvm = ACVM::new(blackbox_solver, circuit.opcodes, initial_witness);

    // Assert messages are not a map due to serde-reflect issues
    let get_assert_message = |opcode_location| {
        circuit
            .assert_messages
            .iter()
            .find(|(loc, _)| loc == opcode_location)
            .map(|(_, message)| message.clone())
    };

    loop {
        let solver_status = acvm.solve();

        match solver_status {
            ACVMStatus::Solved => break,
            ACVMStatus::InProgress => {
                unreachable!("Execution should not stop while in `InProgress` state.")
            }
            ACVMStatus::Failure(error) => {
                return Err(NargoError::ExecutionError(match error {
                    OpcodeResolutionError::UnsatisfiedConstrain {
                        opcode_location: ErrorLocation::Resolved(opcode_location),
                    } => match get_assert_message(&opcode_location) {
                        Some(assert_message) => {
                            ExecutionError::AssertionFailed(assert_message, opcode_location)
                        }
                        None => ExecutionError::SolvingError(error),
                    },
                    _ => ExecutionError::SolvingError(error),
                }))
            }
            ACVMStatus::RequiresForeignCall(foreign_call) => {
                let foreign_call_result = ForeignCall::execute(&foreign_call, show_output)?;
                acvm.resolve_pending_foreign_call(foreign_call_result);
            }
        }
    }

    let solved_witness = acvm.finalize();
    Ok(solved_witness)
}
