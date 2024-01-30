use acvm::brillig_vm::brillig::ForeignCallResult;
use acvm::pwg::{ACVMStatus, ErrorLocation, OpcodeResolutionError, ACVM};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use crate::errors::ExecutionError;
use crate::NargoError;

use super::foreign_calls::{ForeignCallExecutor, NargoForeignCallResult};

#[tracing::instrument(level = "trace", skip_all)]
pub fn execute_circuit<B: BlackBoxFunctionSolver, F: ForeignCallExecutor>(
    circuit: &Circuit,
    initial_witness: WitnessMap,
    blackbox_solver: &B,
    foreign_call_executor: &mut F,
) -> Result<WitnessMap, NargoError> {
    let mut acvm = ACVM::new(blackbox_solver, &circuit.opcodes, initial_witness);
    // TODO: remove only for debugging
    let mut num_foreign_calls = 0;
    let mut assert_message: Option<String> = None;
    loop {
        let solver_status = acvm.solve();
        // dbg!(acvm.get_assert_message());
        match solver_status {
            ACVMStatus::Solved => break,
            ACVMStatus::InProgress => {
                unreachable!("Execution should not stop while in `InProgress` state.")
            }
            ACVMStatus::Failure(error) => {
                dbg!(num_foreign_calls);
                let call_stack = match &error {
                    OpcodeResolutionError::UnsatisfiedConstrain {
                        opcode_location: ErrorLocation::Resolved(opcode_location),
                    } => Some(vec![*opcode_location]),
                    OpcodeResolutionError::BrilligFunctionFailed { call_stack, .. } => {
                        Some(call_stack.clone())
                    }
                    _ => None,
                };

                return Err(NargoError::ExecutionError(match call_stack {
                    Some(call_stack) => {
                        // First check whether we have a runtime assertion message that should be returned on an ACVM failure
                        // If we do not have a runtime assertion message, we should check whether the circuit has any hardcoded
                        // messages associated with a specific `OpcodeLocation`.
                        // Otherwise return the provided opcode resolution error.
                        if let Some(assert_message) = assert_message {
                            ExecutionError::AssertionFailed(assert_message.to_owned(), call_stack)
                        } else if let Some(assert_message) = circuit.get_assert_message(
                            *call_stack.last().expect("Call stacks should not be empty"),
                        ) {
                            ExecutionError::AssertionFailed(assert_message.to_owned(), call_stack)
                        } else {
                            ExecutionError::SolvingError(error)
                        }
                    }
                    None => ExecutionError::SolvingError(error),
                }));
            }
            ACVMStatus::RequiresForeignCall(foreign_call) => {
                let foreign_call_result = foreign_call_executor.execute(&foreign_call)?;
                dbg!(foreign_call_result.clone());
                match foreign_call_result {
                    NargoForeignCallResult::BrilligOutput(foreign_call_result) => {
                        acvm.resolve_pending_foreign_call(foreign_call_result);
                    }
                    NargoForeignCallResult::ResolvedAssertMessage(message) => {
                        if assert_message.is_some() {
                            panic!("ahhh we should not be resolving another assert message as the VM should have failed");
                        }
                        if !message.is_empty() {
                            assert_message = Some(message);
                        }
                        acvm.resolve_pending_foreign_call(ForeignCallResult::default());
                    }
                }
            }
        }
    }

    Ok(acvm.finalize())
}
