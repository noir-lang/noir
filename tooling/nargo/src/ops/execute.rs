use acvm::acir::circuit::OpcodeLocation;
use acvm::pwg::{ACVMStatus, ErrorLocation, OpcodeResolutionError, ACVM};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};
use noirc_printable_type::ForeignCallError;

use crate::errors::ExecutionError;
use crate::NargoError;

use super::foreign_calls::ForeignCallExecutor;

#[tracing::instrument(level = "trace", skip_all)]
pub fn execute_circuit<B: BlackBoxFunctionSolver, F: ForeignCallExecutor>(
    circuit: &Circuit,
    initial_witness: WitnessMap,
    blackbox_solver: &B,
    foreign_call_executor: &mut F,
) -> Result<WitnessMap, NargoError> {
    let mut acvm = ACVM::new(blackbox_solver, &circuit.opcodes, initial_witness);

    let mut err: Option<OpcodeResolutionError> = None;
    loop {

        if let Some(error) = &err {
            let solver_status = if acvm.instruction_pointer() < acvm.opcodes().len() {
                acvm.solve_opcode()
            } else {
                return Err(resolve_comptime_assert_message(error, circuit));
            };
            match solver_status {
                ACVMStatus::RequiresForeignCall(foreign_call) => {
                    let foreign_call_result = foreign_call_executor.execute(&foreign_call)?;

                    let assert_message = foreign_call_result.get_assert_message().expect("Only assert message resolution is supported for execution after an ACVM failure");
                    let call_stack = resolve_call_stack(error);

                    return Err(NargoError::ExecutionError(match call_stack {
                        Some(call_stack) => {
                            ExecutionError::AssertionFailed(assert_message, call_stack)
                        }
                        None => ExecutionError::SolvingError(error.clone()),
                    }));
                }    
                _ => {
                    return Err(resolve_comptime_assert_message(error, circuit))
                }
            }
        } else {
            let solver_status = acvm.solve();

            match solver_status {
                ACVMStatus::Solved => break,
                ACVMStatus::InProgress => {
                    unreachable!("Execution should not stop while in `InProgress` state.")
                }
                ACVMStatus::Failure(error) => {
                    err = Some(error);
                }
                ACVMStatus::RequiresForeignCall(foreign_call) => {
                    let foreign_call_result = foreign_call_executor.execute(&foreign_call)?;
                    acvm.resolve_pending_foreign_call(foreign_call_result);
                }
            }
        } 
    }

    Ok(acvm.finalize())
}

fn resolve_call_stack(error: &OpcodeResolutionError) -> Option<Vec<OpcodeLocation>> {
    match error {
        OpcodeResolutionError::UnsatisfiedConstrain {
            opcode_location: ErrorLocation::Resolved(opcode_location),
        } => Some(vec![*opcode_location]),
        OpcodeResolutionError::BrilligFunctionFailed { call_stack, .. } => {
            Some(call_stack.clone())
        }
        _ => None,
    }
}

fn resolve_comptime_assert_message(error: &OpcodeResolutionError, circuit: &Circuit) -> NargoError {
    let call_stack = resolve_call_stack(error);

    NargoError::ExecutionError(match call_stack {
        Some(call_stack) => {
            if let Some(assert_message) = circuit.get_assert_message(
                *call_stack.last().expect("Call stacks should not be empty"),
            ) {
                ExecutionError::AssertionFailed(assert_message.to_owned(), call_stack)
            } else {
                ExecutionError::SolvingError(error.clone())
            }
        }
        None => ExecutionError::SolvingError(error.clone()),
    })
}
