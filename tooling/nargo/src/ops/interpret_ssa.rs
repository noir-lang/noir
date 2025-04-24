use acvm::acir::circuit::brillig::BrilligBytecode;
use acvm::acir::circuit::{OpcodeLocation, Program};
use acvm::acir::native_types::WitnessStack;
use acvm::brillig_vm::BranchToFeatureMap;
use acvm::pwg::{
    ACVM, ACVMStatus, ErrorLocation, OpcodeNotSolvable, OpcodeResolutionError, ProfilingSamples,
};
use acvm::{AcirField, BlackBoxFunctionSolver};
type NargoErrorAndCoverage<F> = (NargoError<F>, Option<Vec<u32>>);
type WitnessAndCoverage<F> = (WitnessStack<F>, Option<Vec<u32>>);
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use crate::NargoError;
use crate::errors::{ExecutionError, ResolvedOpcodeLocation, execution_error_from};
use crate::foreign_calls::ForeignCallExecutor;

struct ProgramExecutor<'a, F: AcirField, B: BlackBoxFunctionSolver<F>, E: ForeignCallExecutor<F>> {
    functions: &'a [Circuit<F>],

    unconstrained_functions: &'a [BrilligBytecode<F>],

    // This gets built as we run through the program looking at each function call
    witness_stack: WitnessStack<F>,

    blackbox_solver: &'a B,

    foreign_call_executor: &'a mut E,

    // The Noir compiler codegens per function and call stacks are not shared across ACIR function calls.
    // We must rebuild a call stack when executing a program of many circuits.
    call_stack: Vec<ResolvedOpcodeLocation>,

    // Tracks the index of the current function we are executing.
    // This is used to fetch the function we want to execute
    // and to resolve call stack locations across many function calls.
    current_function_index: usize,

    // Flag that states whether we want to profile the VM. Profiling can add extra
    // execution costs so we want to make sure we only trigger it explicitly.
    profiling_active: bool,

    // Flag that states whether we want to trace brillig VM execution
    brillig_fuzzing_active: bool,

    // Brillig branch to feature map
    brillig_branch_to_feature_map: Option<&'a BranchToFeatureMap>,

    // Last recorded fuzzing trace
    last_fuzzing_trace: Option<Vec<u32>>,
}

impl<'a, F: AcirField, B: BlackBoxFunctionSolver<F>, E: ForeignCallExecutor<F>>
    ProgramExecutor<'a, F, B, E>
{
    fn new(
        functions: &'a [Circuit<F>],
        unconstrained_functions: &'a [BrilligBytecode<F>],
        blackbox_solver: &'a B,
        foreign_call_executor: &'a mut E,
        profiling_active: bool,
    ) -> Self {
        ProgramExecutor {
            functions,
            unconstrained_functions,
            witness_stack: WitnessStack::default(),
            blackbox_solver,
            foreign_call_executor,
            call_stack: Vec::default(),
            current_function_index: 0,
            profiling_active,
            brillig_fuzzing_active: false,
            brillig_branch_to_feature_map: None,
            last_fuzzing_trace: None,
        }
    }

    fn with_brillig_fuzzing(
        &mut self,
        brillig_branch_to_feature_map: Option<&'a BranchToFeatureMap>,
    ) {
        self.brillig_fuzzing_active = brillig_branch_to_feature_map.is_some();
        self.brillig_branch_to_feature_map = brillig_branch_to_feature_map;
    }

    fn finalize(self) -> WitnessStack<F> {
        self.witness_stack
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn execute_circuit(
        &mut self,
        initial_witness: WitnessMap<F>,
    ) -> Result<(WitnessMap<F>, ProfilingSamples), NargoError<F>> {
        let circuit = &self.functions[self.current_function_index];
        let mut acvm = ACVM::new(
            self.blackbox_solver,
            &circuit.opcodes,
            initial_witness,
            self.unconstrained_functions,
            &circuit.assert_messages,
        );
        acvm.with_profiler(self.profiling_active);
        acvm.with_brillig_fuzzing(self.brillig_branch_to_feature_map);

        loop {
            let solver_status = acvm.solve();

            match solver_status {
                ACVMStatus::Solved => break,
                ACVMStatus::InProgress => {
                    unreachable!("Execution should not stop while in `InProgress` state.")
                }
                ACVMStatus::Failure(error) => {
                    self.last_fuzzing_trace = acvm.get_brillig_fuzzing_trace();
                    match &error {
                        OpcodeResolutionError::UnsatisfiedConstrain {
                            opcode_location: ErrorLocation::Resolved(opcode_location),
                            ..
                        }
                        | OpcodeResolutionError::IndexOutOfBounds {
                            opcode_location: ErrorLocation::Resolved(opcode_location),
                            ..
                        }
                        | OpcodeResolutionError::InvalidInputBitSize {
                            opcode_location: ErrorLocation::Resolved(opcode_location),
                            ..
                        } => {
                            let resolved_location = ResolvedOpcodeLocation {
                                acir_function_index: self.current_function_index,
                                opcode_location: *opcode_location,
                            };
                            self.call_stack.push(resolved_location);
                        }
                        OpcodeResolutionError::BrilligFunctionFailed { call_stack, .. } => {
                            let brillig_call_stack =
                                call_stack.iter().map(|location| ResolvedOpcodeLocation {
                                    acir_function_index: self.current_function_index,
                                    opcode_location: *location,
                                });
                            self.call_stack.extend(brillig_call_stack);
                        }
                        _ => (),
                    };

                    return Err(NargoError::ExecutionError(execution_error_from(
                        error,
                        &self.call_stack,
                    )));
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
                    self.current_function_index = call_info.id.as_usize();
                    // Execute the ACIR call
                    let acir_to_call = &self.functions[call_info.id.as_usize()];
                    let initial_witness = call_info.initial_witness;
                    // TODO: Profiling among multiple circuits is not supported
                    let (call_solved_witness, _) = self.execute_circuit(initial_witness)?;

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
                    self.witness_stack.push(call_info.id.0, call_solved_witness);
                }
            }
        }
        // Clear the call stack if we have succeeded in executing the circuit.
        // This needs to be done or else all successful ACIR call stacks will also be
        // included in a failure case.
        self.call_stack.clear();

        let profiling_samples = acvm.take_profiling_samples();
        self.last_fuzzing_trace = acvm.get_brillig_fuzzing_trace();
        Ok((acvm.finalize(), profiling_samples))
    }
}

pub fn execute_program<F: AcirField, B: BlackBoxFunctionSolver<F>, E: ForeignCallExecutor<F>>(
    program: &Program<F>,
    initial_witness: WitnessMap<F>,
    blackbox_solver: &B,
    foreign_call_executor: &mut E,
) -> Result<WitnessStack<F>, NargoError<F>> {
    let profiling_active = false;
    let (witness_stack, profiling_samples) = execute_program_inner(
        program,
        initial_witness,
        blackbox_solver,
        foreign_call_executor,
        profiling_active,
    )?;
    assert!(profiling_samples.is_empty(), "Expected no profiling samples");

    Ok(witness_stack)
}

pub fn execute_program_with_profiling<
    F: AcirField,
    B: BlackBoxFunctionSolver<F>,
    E: ForeignCallExecutor<F>,
>(
    program: &Program<F>,
    initial_witness: WitnessMap<F>,
    blackbox_solver: &B,
    foreign_call_executor: &mut E,
) -> Result<(WitnessStack<F>, ProfilingSamples), NargoError<F>> {
    let profiling_active = true;
    execute_program_inner(
        program,
        initial_witness,
        blackbox_solver,
        foreign_call_executor,
        profiling_active,
    )
}
pub(crate) fn execute_program_with_brillig_fuzzing<
    F: AcirField,
    B: BlackBoxFunctionSolver<F>,
    E: ForeignCallExecutor<F>,
>(
    program: &Program<F>,
    initial_witness: WitnessMap<F>,
    blackbox_solver: &B,
    foreign_call_executor: &mut E,
    brillig_branch_to_feature_map: Option<&BranchToFeatureMap>,
) -> Result<WitnessAndCoverage<F>, NargoErrorAndCoverage<F>> {
    let mut executor = ProgramExecutor::new(
        &program.functions,
        &program.unconstrained_functions,
        blackbox_solver,
        foreign_call_executor,
        false,
    );
    assert!(brillig_branch_to_feature_map.is_some());
    executor.with_brillig_fuzzing(brillig_branch_to_feature_map);
    match executor.execute_circuit(initial_witness) {
        Ok((main_witness, _)) => {
            executor.witness_stack.push(0, main_witness);
            let trace = executor.last_fuzzing_trace.clone();
            Ok((executor.finalize(), trace))
        }
        Err(err) => Err((err, executor.last_fuzzing_trace)),
    }
}
#[tracing::instrument(level = "trace", skip_all)]
fn execute_program_inner<F: AcirField, B: BlackBoxFunctionSolver<F>, E: ForeignCallExecutor<F>>(
    program: &Program<F>,
    initial_witness: WitnessMap<F>,
    blackbox_solver: &B,
    foreign_call_executor: &mut E,
    profiling_active: bool,
) -> Result<(WitnessStack<F>, ProfilingSamples), NargoError<F>> {
    let mut executor = ProgramExecutor::new(
        &program.functions,
        &program.unconstrained_functions,
        blackbox_solver,
        foreign_call_executor,
        profiling_active,
    );
    let (main_witness, profiling_samples) = executor.execute_circuit(initial_witness)?;
    executor.witness_stack.push(0, main_witness);

    Ok((executor.finalize(), profiling_samples))
}
