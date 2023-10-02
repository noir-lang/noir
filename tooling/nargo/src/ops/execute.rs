use acvm::acir::circuit::OpcodeLocation;
use acvm::pwg::{ACVMStatus, ErrorLocation, OpcodeResolutionError, ACVM};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};
use noirc_errors::{debug_info::DebugInfo, Location};
use noirc_errors::{CustomDiagnostic, FileDiagnostic};

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

    // Assert messages are not a map due to https://github.com/noir-lang/acvm/issues/522
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
                        if let Some(assert_message) = get_assert_message(
                            call_stack.last().expect("Call stacks should not be empty"),
                        ) {
                            ExecutionError::AssertionFailed(assert_message, call_stack)
                        } else {
                            ExecutionError::SolvingError(error)
                        }
                    }
                    None => ExecutionError::SolvingError(error),
                }));
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

fn extract_locations_from_error(
    error: &ExecutionError,
    debug: &DebugInfo,
) -> Option<Vec<Location>> {
    let mut opcode_locations = match error {
        ExecutionError::SolvingError(OpcodeResolutionError::BrilligFunctionFailed {
            call_stack,
            ..
        })
        | ExecutionError::AssertionFailed(_, call_stack) => Some(call_stack.clone()),
        ExecutionError::SolvingError(OpcodeResolutionError::IndexOutOfBounds {
            opcode_location: error_location,
            ..
        })
        | ExecutionError::SolvingError(OpcodeResolutionError::UnsatisfiedConstrain {
            opcode_location: error_location,
        }) => match error_location {
            ErrorLocation::Unresolved => {
                unreachable!("Cannot resolve index for unsatisfied constraint")
            }
            ErrorLocation::Resolved(opcode_location) => Some(vec![*opcode_location]),
        },
        _ => None,
    }?;

    if let Some(OpcodeLocation::Brillig { acir_index, .. }) = opcode_locations.get(0) {
        opcode_locations.insert(0, OpcodeLocation::Acir(*acir_index));
    }

    Some(
        opcode_locations
            .iter()
            .flat_map(|opcode_location| debug.opcode_location(opcode_location).unwrap_or_default())
            .collect(),
    )
}

pub fn try_to_diagnose_error(nargo_err: &NargoError, debug: &DebugInfo) -> Option<FileDiagnostic> {
    if let NargoError::ExecutionError(execution_error) = nargo_err {
        if let Some(source_locations) = extract_locations_from_error(execution_error, debug) {
            // The location of the error itself will be the location at the top
            // of the call stack (the last item in the Vec).
            if let Some(location) = source_locations.last() {
                let message = match nargo_err {
                    NargoError::ExecutionError(ExecutionError::AssertionFailed(message, _)) => {
                        format!("Assertion failed: '{message}'")
                    }
                    NargoError::ExecutionError(ExecutionError::SolvingError(
                        OpcodeResolutionError::IndexOutOfBounds { index, array_size, .. },
                    )) => {
                        format!(
                                "Index out of bounds, array has size {array_size:?}, but index was {index:?}"
                            )
                    }
                    NargoError::ExecutionError(ExecutionError::SolvingError(
                        OpcodeResolutionError::UnsatisfiedConstrain { .. },
                    )) => "Failed constraint".into(),
                    _ => nargo_err.to_string(),
                };
                return Some(
                    CustomDiagnostic::simple_error(message, String::new(), location.span)
                        .in_file(location.file)
                        .with_call_stack(source_locations),
                );
            }
        }
    }
    None
}
