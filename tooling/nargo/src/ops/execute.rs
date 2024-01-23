use acvm::acir::circuit::OpcodeLocation;
use acvm::pwg::{ACVMForeignCallResult, ACVMStatus, ErrorLocation, OpcodeResolutionError, ACVM};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

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
                        if let Some(assert_message) = acvm.get_assert_message() {
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
                acvm.resolve_pending_foreign_call(foreign_call_result);
            }
        }
    }

    Ok(acvm.finalize())
}

fn resolve_call_stack(error: &OpcodeResolutionError) -> Option<(Vec<OpcodeLocation>, bool)> {
    match error {
        OpcodeResolutionError::UnsatisfiedConstrain {
            opcode_location: ErrorLocation::Resolved(opcode_location),
        } => Some((vec![*opcode_location], false)),
        OpcodeResolutionError::BrilligFunctionFailed { call_stack, .. } => {
            dbg!("got brillig func failed");
            Some((call_stack.clone(), true))
        }
        _ => None,
    }
}

fn resolve_comptime_assert_message(error: &OpcodeResolutionError, circuit: &Circuit) -> NargoError {
    let call_stack= resolve_call_stack(error);

    NargoError::ExecutionError(match call_stack {
        Some((call_stack, _)) => {
            if let Some(assert_message) = circuit
                .get_assert_message(*call_stack.last().expect("Call stacks should not be empty"))
            {
                ExecutionError::AssertionFailed(assert_message.to_owned(), call_stack)
            } else {
                ExecutionError::SolvingError(error.clone())
            }
        }
        None => ExecutionError::SolvingError(error.clone()),
    })
}

// fn can_process_opcode_after_failure<'a, B: BlackBoxFunctionSolver>(acvm: &ACVM<'a, B>) -> bool {
//     if acvm.instruction_pointer() >= acvm.opcodes().len() {
//         return false;
//     }
//     if let Opcode::Brillig(brillig) = &acvm.opcodes()[acvm.instruction_pointer()] {
//         // We do not want 
//         match &brillig.bytecode[brillig.bytecode.len() - 2] {
//             acvm::brillig_vm::brillig::Opcode::ForeignCall { function, .. } => {
//                 ForeignCall::execution_allowed_after_failure(function)
//             }
//             _ => false,
//         }
//     } else {
//         false
//     }
// }
