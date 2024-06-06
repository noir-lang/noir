use acvm::acir::circuit::brillig::BrilligBytecode;
use acvm::acir::circuit::{
    OpcodeLocation, Program, ResolvedAssertionPayload, ResolvedOpcodeLocation,
};
use acvm::acir::native_types::WitnessStack;
use acvm::pwg::{ACVMStatus, ErrorLocation, OpcodeNotSolvable, OpcodeResolutionError, ACVM};
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};
use acvm::{BlackBoxFunctionSolver, FieldElement};

use crate::errors::ExecutionError;
use crate::NargoError;

use super::foreign_calls::ForeignCallExecutor;

struct ProgramExecutor<'a, B: BlackBoxFunctionSolver<FieldElement>, F: ForeignCallExecutor> {
    functions: &'a [Circuit<FieldElement>],

    unconstrained_functions: &'a [BrilligBytecode<FieldElement>],

    // This gets built as we run through the program looking at each function call
    witness_stack: WitnessStack<FieldElement>,

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

impl<'a, B: BlackBoxFunctionSolver<FieldElement>, F: ForeignCallExecutor>
    ProgramExecutor<'a, B, F>
{
    fn new(
        functions: &'a [Circuit<FieldElement>],
        unconstrained_functions: &'a [BrilligBytecode<FieldElement>],
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

    fn finalize(self) -> WitnessStack<FieldElement> {
        self.witness_stack
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn execute_circuit(
        &mut self,
        initial_witness: WitnessMap<FieldElement>,
    ) -> Result<WitnessMap<FieldElement>, NargoError> {
        let circuit = &self.functions[self.current_function_index];
        let mut acvm = ACVM::new(
            self.blackbox_solver,
            &circuit.opcodes,
            initial_witness,
            self.unconstrained_functions,
            &circuit.assert_messages,
        );

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
                            ..
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

                    let assertion_payload: Option<ResolvedAssertionPayload<FieldElement>> =
                        match &error {
                            OpcodeResolutionError::BrilligFunctionFailed { payload, .. }
                            | OpcodeResolutionError::UnsatisfiedConstrain { payload, .. } => {
                                payload.clone()
                            }
                            _ => None,
                        };

                    return Err(NargoError::ExecutionError(match assertion_payload {
                        Some(payload) => ExecutionError::AssertionFailed(
                            payload,
                            call_stack.expect("Should have call stack for an assertion failure"),
                        ),
                        None => ExecutionError::SolvingError(error, call_stack),
                    }));
                }
                ACVMStatus::RequiresForeignCall(foreign_call) => {
                    let foreign_call_result = self.foreign_call_executor.execute(&foreign_call)?;
                    acvm.resolve_pending_foreign_call(foreign_call_result);
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
pub fn execute_program<B: BlackBoxFunctionSolver<FieldElement>, F: ForeignCallExecutor>(
    program: &Program<FieldElement>,
    initial_witness: WitnessMap<FieldElement>,
    blackbox_solver: &B,
    foreign_call_executor: &mut F,
) -> Result<WitnessStack<FieldElement>, NargoError> {
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
