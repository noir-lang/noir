use acvm::acir::circuit::{OpcodeLocation, Opcode};
use acvm::pwg::{ACVMStatus, ErrorLocation, OpcodeResolutionError, ACVM};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use crate::errors::ExecutionError;
use crate::NargoError;

use super::foreign_calls::{ForeignCallExecutor, ForeignCall};

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
            // If there are two assertions in a row and the second one is false we could hit 
            // a failure status that will resolve a comptime assert message rather than a runtime assert
            // message as we are expecting.
            // If there is a Brillig assertion we are just going to process the next Brillig func rather than 
            // 
            dbg!("got err");
            // dbg!(&acvm.opcodes()[acvm.instruction_pointer()]);
            // dbg!(&acvm.opcodes()[acvm.instruction_pointer() - 1]);
            // dbg!(&acvm.opcodes()[acvm.instruction_pointer() - 2]);

            let call_stack = resolve_call_stack(error);

            // Consrtuct error 
            match call_stack.clone() {
                Some((call_stack, is_brillig_fail)) => {
                    if is_brillig_fail {
                        dbg!("got brillig fail");
                        let x = acvm.step_into_brillig_opcode();
                    }
                }
                None => {
                }
            }

            let solver_status = if can_process_opcode_after_failure(&acvm) {
                acvm.solve_opcode()
            } else {
                return Err(resolve_comptime_assert_message(error, circuit));
            };
            // dbg!(solver_status.clone());
            match solver_status {
                ACVMStatus::RequiresForeignCall(foreign_call) => {
                    let foreign_call_result = foreign_call_executor.execute(&foreign_call)?;

                    let assert_message = foreign_call_result.get_assert_message().expect("Only assert message resolution is supported for execution after an ACVM failure");

                    return Err(NargoError::ExecutionError(match call_stack {
                        Some((call_stack, is_brillig_fail)) => {
                            ExecutionError::AssertionFailed(assert_message, call_stack)
                        }
                        None => ExecutionError::SolvingError(error.clone()),
                    }));
                }
                _ => return Err(resolve_comptime_assert_message(error, circuit)),
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
                    // dbg!(foreign_call.clone());
                    let foreign_call_result = foreign_call_executor.execute(&foreign_call)?;
                    acvm.resolve_pending_foreign_call(foreign_call_result);
                }
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

fn can_process_opcode_after_failure<'a, B: BlackBoxFunctionSolver>(acvm: &ACVM<'a, B>) -> bool {
    if acvm.instruction_pointer() >= acvm.opcodes().len() {
        return false;
    }
    if let Opcode::Brillig(brillig) = &acvm.opcodes()[acvm.instruction_pointer()] {
        // We do not want 
        match &brillig.bytecode[brillig.bytecode.len() - 2] {
            acvm::brillig_vm::brillig::Opcode::ForeignCall { function, .. } => {
                ForeignCall::execution_allowed_after_failure(function)
            }
            _ => false,
        }
        // if matches!(&brillig.bytecode[brillig.bytecode.len() - 2], acvm::brillig_vm::brillig::Opcode::ForeignCall { function, .. }) {
        //     // if function
        //     // if function
        //     true
        // } else {
        //     false
        // }
    } else {
        false
    }
}
