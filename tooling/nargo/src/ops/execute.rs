use acvm::acir::circuit::Program;
use acvm::acir::native_types::WitnessStack;
use acvm::brillig_vm::brillig::ForeignCallResult;
use acvm::pwg::{ACVMStatus, ErrorLocation, OpcodeNotSolvable, OpcodeResolutionError, ACVM};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use crate::errors::ExecutionError;
use crate::NargoError;

use super::foreign_calls::{ForeignCallExecutor, NargoForeignCallResult};

struct ProgramExecutor<'a, B: BlackBoxFunctionSolver, F: ForeignCallExecutor> {
    functions: &'a [Circuit],
    // This gets built as we run through the program looking at each function call
    witness_stack: WitnessStack,

    blackbox_solver: &'a B,

    foreign_call_executor: &'a mut F,
}

impl<'a, B: BlackBoxFunctionSolver, F: ForeignCallExecutor> ProgramExecutor<'a, B, F> {
    fn new(
        functions: &'a [Circuit],
        blackbox_solver: &'a B,
        foreign_call_executor: &'a mut F,
    ) -> Self {
        ProgramExecutor {
            functions,
            witness_stack: WitnessStack::default(),
            blackbox_solver,
            foreign_call_executor,
        }
    }

    fn finalize(self) -> WitnessStack {
        self.witness_stack
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn execute_circuit(
        &mut self,
        circuit: &Circuit,
        initial_witness: WitnessMap,
    ) -> Result<WitnessMap, NargoError> {
        let mut acvm = ACVM::new(self.blackbox_solver, &circuit.opcodes, initial_witness);

        // This message should be resolved by a nargo foreign call only when we have an unsatisfied assertion.
        let mut assert_message: Option<String> = None;
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
                            // First check whether we have a runtime assertion message that should be resolved on an ACVM failure
                            // If we do not have a runtime assertion message, we check wether the error is a brillig error with a user-defined message,
                            // and finally we should check whether the circuit has any hardcoded messages associated with a specific `OpcodeLocation`.
                            // Otherwise return the provided opcode resolution error.
                            if let Some(assert_message) = assert_message {
                                ExecutionError::AssertionFailed(
                                    assert_message.to_owned(),
                                    call_stack,
                                )
                            } else if let OpcodeResolutionError::BrilligFunctionFailed {
                                message: Some(message),
                                ..
                            } = &error
                            {
                                ExecutionError::AssertionFailed(message.to_owned(), call_stack)
                            } else if let Some(assert_message) = circuit.get_assert_message(
                                *call_stack.last().expect("Call stacks should not be empty"),
                            ) {
                                ExecutionError::AssertionFailed(
                                    assert_message.to_owned(),
                                    call_stack,
                                )
                            } else {
                                ExecutionError::SolvingError(error)
                            }
                        }
                        None => ExecutionError::SolvingError(error),
                    }));
                }
                ACVMStatus::RequiresForeignCall(foreign_call) => {
                    let foreign_call_result = self.foreign_call_executor.execute(&foreign_call)?;
                    match foreign_call_result {
                        NargoForeignCallResult::BrilligOutput(foreign_call_result) => {
                            acvm.resolve_pending_foreign_call(foreign_call_result);
                        }
                        NargoForeignCallResult::ResolvedAssertMessage(message) => {
                            if assert_message.is_some() {
                                unreachable!("Resolving an assert message should happen only once as the VM should have failed");
                            }
                            assert_message = Some(message);

                            acvm.resolve_pending_foreign_call(ForeignCallResult::default());
                        }
                    }
                }
                ACVMStatus::RequiresAcirCall(call_info) => {
                    let acir_to_call = &self.functions[call_info.id as usize];
                    let initial_witness = call_info.initial_witness;
                    let call_solved_witness =
                        self.execute_circuit(acir_to_call, initial_witness)?;
                    let mut call_resolved_outputs = Vec::new();
                    for return_witness_index in acir_to_call.return_values.indices() {
                        if let Some(return_value) =
                            call_solved_witness.get_index(return_witness_index)
                        {
                            call_resolved_outputs.push(*return_value);
                        } else {
                            return Err(ExecutionError::SolvingError(
                                OpcodeNotSolvable::MissingAssignment(return_witness_index).into(),
                            )
                            .into());
                        }
                    }
                    acvm.resolve_pending_acir_call(call_resolved_outputs);
                    self.witness_stack.push(call_info.id, call_solved_witness);
                }
            }
        }

        Ok(acvm.finalize())
    }
}

#[tracing::instrument(level = "trace", skip_all)]
pub fn execute_program<B: BlackBoxFunctionSolver, F: ForeignCallExecutor>(
    program: &Program,
    initial_witness: WitnessMap,
    blackbox_solver: &B,
    foreign_call_executor: &mut F,
) -> Result<WitnessStack, NargoError> {
    let main = &program.functions[0];

    let mut executor =
        ProgramExecutor::new(&program.functions, blackbox_solver, foreign_call_executor);
    let main_witness = executor.execute_circuit(main, initial_witness)?;
    executor.witness_stack.push(0, main_witness);

    Ok(executor.finalize())
}
