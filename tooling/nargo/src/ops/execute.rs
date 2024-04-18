use acvm::acir::circuit::brillig::BrilligBytecode;
use acvm::acir::circuit::{OpcodeLocation, Program, ResolvedOpcodeLocation};
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

    unconstrained_functions: &'a [BrilligBytecode],

    // This gets built as we run through the program looking at each function call
    witness_stack: WitnessStack,

    blackbox_solver: &'a B,

    foreign_call_executor: &'a mut F,

    // The Noir compiler codegens per function and call stacks are not shared across ACIR function calls.
    // We must rebuild a call stack when executing a program of many circuits.
    call_stack: Vec<ResolvedOpcodeLocation>,

    // Tracks the index of the current function we are executing.
    // This is used to fetch the function we want to execute
    // and to resolve call stack locations across many function calls.
    current_function_index: usize,
}

impl<'a, B: BlackBoxFunctionSolver, F: ForeignCallExecutor> ProgramExecutor<'a, B, F> {
    fn new(
        functions: &'a [Circuit],
        unconstrained_functions: &'a [BrilligBytecode],
        blackbox_solver: &'a B,
        foreign_call_executor: &'a mut F,
    ) -> Self {
        ProgramExecutor {
            functions,
            unconstrained_functions,
            witness_stack: WitnessStack::default(),
            blackbox_solver,
            foreign_call_executor,
            call_stack: Vec::default(),
            current_function_index: 0,
        }
    }

    fn finalize(self) -> WitnessStack {
        self.witness_stack
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn execute_circuit(&mut self, initial_witness: WitnessMap) -> Result<WitnessMap, NargoError> {
        let circuit = &self.functions[self.current_function_index];
        let mut acvm = ACVM::new(
            self.blackbox_solver,
            &circuit.opcodes,
            initial_witness,
            self.unconstrained_functions,
        );

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
                        }
                        | OpcodeResolutionError::IndexOutOfBounds {
                            opcode_location: ErrorLocation::Resolved(opcode_location),
                            ..
                        } => {
                            let resolved_location = ResolvedOpcodeLocation {
                                acir_function_index: self.current_function_index,
                                opcode_location: *opcode_location,
                            };
                            self.call_stack.push(resolved_location);
                            Some(self.call_stack.clone())
                        }
                        OpcodeResolutionError::BrilligFunctionFailed { call_stack, .. } => {
                            let brillig_call_stack =
                                call_stack.iter().map(|location| ResolvedOpcodeLocation {
                                    acir_function_index: self.current_function_index,
                                    opcode_location: *location,
                                });
                            self.call_stack.extend(brillig_call_stack);
                            Some(self.call_stack.clone())
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
                                call_stack
                                    .last()
                                    .expect("Call stacks should not be empty")
                                    .opcode_location,
                            ) {
                                ExecutionError::AssertionFailed(
                                    assert_message.to_owned(),
                                    call_stack,
                                )
                            } else {
                                ExecutionError::SolvingError(error, Some(call_stack))
                            }
                        }
                        None => ExecutionError::SolvingError(error, None),
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
                    // Store the parent function index whose context we are currently executing
                    let acir_function_caller = self.current_function_index;
                    // Add call opcode to the call stack with a reference to the parent function index
                    self.call_stack.push(ResolvedOpcodeLocation {
                        acir_function_index: acir_function_caller,
                        opcode_location: OpcodeLocation::Acir(acvm.instruction_pointer()),
                    });

                    // Set current function to the circuit we are about to execute
                    self.current_function_index = call_info.id as usize;
                    // Execute the ACIR call
                    let acir_to_call = &self.functions[call_info.id as usize];
                    let initial_witness = call_info.initial_witness;
                    let call_solved_witness = self.execute_circuit(initial_witness)?;

                    // Set tracking index back to the parent function after ACIR call execution
                    self.current_function_index = acir_function_caller;

                    let mut call_resolved_outputs = Vec::new();
                    for return_witness_index in acir_to_call.return_values.indices() {
                        if let Some(return_value) =
                            call_solved_witness.get_index(return_witness_index)
                        {
                            call_resolved_outputs.push(*return_value);
                        } else {
                            return Err(ExecutionError::SolvingError(
                                OpcodeNotSolvable::MissingAssignment(return_witness_index).into(),
                                None, // Missing assignment errors do not supply user-facing diagnostics so we do not need to attach a call stack
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
    let mut executor = ProgramExecutor::new(
        &program.functions,
        &program.unconstrained_functions,
        blackbox_solver,
        foreign_call_executor,
    );
    let main_witness = executor.execute_circuit(initial_witness)?;
    executor.witness_stack.push(0, main_witness);

    Ok(executor.finalize())
}
