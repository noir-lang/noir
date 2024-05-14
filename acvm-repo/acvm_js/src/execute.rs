use std::{future::Future, pin::Pin};

use acvm::acir::circuit::brillig::BrilligBytecode;
use acvm::acir::circuit::ResolvedAssertionPayload;
use acvm::BlackBoxFunctionSolver;
use acvm::{
    acir::circuit::{Circuit, Program},
    acir::native_types::{WitnessMap, WitnessStack},
    pwg::{ACVMStatus, ErrorLocation, OpcodeResolutionError, ACVM},
};
use bn254_blackbox_solver::Bn254BlackBoxSolver;

use js_sys::Error;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    foreign_call::{resolve_brillig, ForeignCallHandler},
    public_witness::extract_indices,
    JsExecutionError, JsSolvedAndReturnWitness, JsWitnessMap, JsWitnessStack,
};

#[wasm_bindgen]
pub struct WasmBlackBoxFunctionSolver(Bn254BlackBoxSolver);

impl WasmBlackBoxFunctionSolver {
    async fn initialize() -> WasmBlackBoxFunctionSolver {
        WasmBlackBoxFunctionSolver(Bn254BlackBoxSolver::initialize().await)
    }
}

#[wasm_bindgen(js_name = "createBlackBoxSolver")]
pub async fn create_black_box_solver() -> WasmBlackBoxFunctionSolver {
    WasmBlackBoxFunctionSolver::initialize().await
}

/// Executes an ACIR circuit to generate the solved witness from the initial witness.
///
/// @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
/// @param {WitnessMap} initial_witness - The initial witness map defining all of the inputs to `circuit`..
/// @param {ForeignCallHandler} foreign_call_handler - A callback to process any foreign calls from the circuit.
/// @returns {WitnessMap} The solved witness calculated by executing the circuit on the provided inputs.
#[wasm_bindgen(js_name = executeCircuit, skip_jsdoc)]
pub async fn execute_circuit(
    program: Vec<u8>,
    initial_witness: JsWitnessMap,
    foreign_call_handler: ForeignCallHandler,
) -> Result<JsWitnessMap, Error> {
    console_error_panic_hook::set_once();

    let solver = WasmBlackBoxFunctionSolver::initialize().await;

    let mut witness_stack = execute_program_with_native_type_return(
        &solver,
        program,
        initial_witness,
        &foreign_call_handler,
    )
    .await?;
    let witness_map =
        witness_stack.pop().expect("Should have at least one witness on the stack").witness;
    Ok(witness_map.into())
}

/// Executes an ACIR circuit to generate the solved witness from the initial witness.
/// This method also extracts the public return values from the solved witness into its own return witness.
///
/// @param {&WasmBlackBoxFunctionSolver} solver - A black box solver.
/// @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
/// @param {WitnessMap} initial_witness - The initial witness map defining all of the inputs to `circuit`..
/// @param {ForeignCallHandler} foreign_call_handler - A callback to process any foreign calls from the circuit.
/// @returns {SolvedAndReturnWitness} The solved witness calculated by executing the circuit on the provided inputs, as well as the return witness indices as specified by the circuit.
#[wasm_bindgen(js_name = executeCircuitWithReturnWitness, skip_jsdoc)]
pub async fn execute_circuit_with_return_witness(
    solver: &WasmBlackBoxFunctionSolver,
    program: Vec<u8>,
    initial_witness: JsWitnessMap,
    foreign_call_handler: ForeignCallHandler,
) -> Result<JsSolvedAndReturnWitness, Error> {
    console_error_panic_hook::set_once();

    let program: Program = Program::deserialize_program(&program)
    .map_err(|_| JsExecutionError::new("Failed to deserialize circuit. This is likely due to differing serialization formats between ACVM_JS and your compiler".to_string(), None, None))?;

    let mut witness_stack = execute_program_with_native_program_and_return(
        solver,
        &program,
        initial_witness,
        &foreign_call_handler,
    )
    .await?;
    let solved_witness =
        witness_stack.pop().expect("Should have at least one witness on the stack").witness;

    let main_circuit = &program.functions[0];
    let return_witness =
        extract_indices(&solved_witness, main_circuit.return_values.0.iter().copied().collect())
            .map_err(|err| JsExecutionError::new(err, None, None))?;

    Ok((solved_witness, return_witness).into())
}

/// Executes an ACIR circuit to generate the solved witness from the initial witness.
///
/// @param {&WasmBlackBoxFunctionSolver} solver - A black box solver.
/// @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
/// @param {WitnessMap} initial_witness - The initial witness map defining all of the inputs to `circuit`..
/// @param {ForeignCallHandler} foreign_call_handler - A callback to process any foreign calls from the circuit.
/// @returns {WitnessMap} The solved witness calculated by executing the circuit on the provided inputs.
#[wasm_bindgen(js_name = executeCircuitWithBlackBoxSolver, skip_jsdoc)]
pub async fn execute_circuit_with_black_box_solver(
    solver: &WasmBlackBoxFunctionSolver,
    program: Vec<u8>,
    initial_witness: JsWitnessMap,
    foreign_call_handler: ForeignCallHandler,
) -> Result<JsWitnessMap, Error> {
    console_error_panic_hook::set_once();

    let mut witness_stack = execute_program_with_native_type_return(
        solver,
        program,
        initial_witness,
        &foreign_call_handler,
    )
    .await?;
    let witness_map =
        witness_stack.pop().expect("Should have at least one witness on the stack").witness;
    Ok(witness_map.into())
}

#[wasm_bindgen(js_name = executeProgram, skip_jsdoc)]
pub async fn execute_program(
    program: Vec<u8>,
    initial_witness: JsWitnessMap,
    foreign_call_handler: ForeignCallHandler,
) -> Result<JsWitnessStack, Error> {
    console_error_panic_hook::set_once();

    let solver = WasmBlackBoxFunctionSolver::initialize().await;

    execute_program_with_black_box_solver(&solver, program, initial_witness, &foreign_call_handler)
        .await
}

#[wasm_bindgen(js_name = executeProgramWithBlackBoxSolver, skip_jsdoc)]
pub async fn execute_program_with_black_box_solver(
    solver: &WasmBlackBoxFunctionSolver,
    program: Vec<u8>,
    initial_witness: JsWitnessMap,
    foreign_call_executor: &ForeignCallHandler,
) -> Result<JsWitnessStack, Error> {
    let witness_stack = execute_program_with_native_type_return(
        solver,
        program,
        initial_witness,
        foreign_call_executor,
    )
    .await?;

    Ok(witness_stack.into())
}

async fn execute_program_with_native_type_return(
    solver: &WasmBlackBoxFunctionSolver,
    program: Vec<u8>,
    initial_witness: JsWitnessMap,
    foreign_call_executor: &ForeignCallHandler,
) -> Result<WitnessStack, Error> {
    let program: Program = Program::deserialize_program(&program)
    .map_err(|_| JsExecutionError::new(
        "Failed to deserialize circuit. This is likely due to differing serialization formats between ACVM_JS and your compiler".to_string(), 
        None,
        None))?;

    execute_program_with_native_program_and_return(
        solver,
        &program,
        initial_witness,
        foreign_call_executor,
    )
    .await
}

async fn execute_program_with_native_program_and_return(
    solver: &WasmBlackBoxFunctionSolver,
    program: &Program,
    initial_witness: JsWitnessMap,
    foreign_call_executor: &ForeignCallHandler,
) -> Result<WitnessStack, Error> {
    let executor = ProgramExecutor::new(
        &program.functions,
        &program.unconstrained_functions,
        &solver.0,
        foreign_call_executor,
    );
    let witness_stack = executor.execute(initial_witness.into()).await?;

    Ok(witness_stack)
}

struct ProgramExecutor<'a, B: BlackBoxFunctionSolver> {
    functions: &'a [Circuit],

    unconstrained_functions: &'a [BrilligBytecode],

    blackbox_solver: &'a B,

    foreign_call_handler: &'a ForeignCallHandler,
}

impl<'a, B: BlackBoxFunctionSolver> ProgramExecutor<'a, B> {
    fn new(
        functions: &'a [Circuit],
        unconstrained_functions: &'a [BrilligBytecode],
        blackbox_solver: &'a B,
        foreign_call_handler: &'a ForeignCallHandler,
    ) -> Self {
        ProgramExecutor {
            functions,
            unconstrained_functions,
            blackbox_solver,
            foreign_call_handler,
        }
    }

    async fn execute(&self, initial_witness: WitnessMap) -> Result<WitnessStack, Error> {
        let main = &self.functions[0];

        let mut witness_stack = WitnessStack::default();
        let main_witness = self.execute_circuit(main, initial_witness, &mut witness_stack).await?;
        witness_stack.push(0, main_witness);
        Ok(witness_stack)
    }

    fn execute_circuit(
        &'a self,
        circuit: &'a Circuit,
        initial_witness: WitnessMap,
        witness_stack: &'a mut WitnessStack,
    ) -> Pin<Box<dyn Future<Output = Result<WitnessMap, Error>> + 'a>> {
        Box::pin(async {
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
                        // Fetch call stack
                        let call_stack = match &error {
                            OpcodeResolutionError::UnsatisfiedConstrain {
                                opcode_location: ErrorLocation::Resolved(opcode_location),
                                ..
                            }
                            | OpcodeResolutionError::IndexOutOfBounds {
                                opcode_location: ErrorLocation::Resolved(opcode_location),
                                ..
                            } => Some(vec![*opcode_location]),
                            OpcodeResolutionError::BrilligFunctionFailed { call_stack, .. } => {
                                Some(call_stack.clone())
                            }
                            _ => None,
                        };
                        // If the failed opcode has an assertion message, integrate it into the error message for backwards compatibility.
                        // Otherwise, pass the raw assertion payload as is.
                        let (message, raw_assertion_payload) = match error {
                            OpcodeResolutionError::UnsatisfiedConstrain {
                                payload: Some(payload),
                                ..
                            }
                            | OpcodeResolutionError::BrilligFunctionFailed {
                                payload: Some(payload),
                                ..
                            } => match payload {
                                ResolvedAssertionPayload::Raw(raw_payload) => {
                                    ("Assertion failed".to_string(), Some(raw_payload))
                                }
                                ResolvedAssertionPayload::String(message) => {
                                    (format!("Assertion failed: {}", message), None)
                                }
                            },
                            _ => (error.to_string(), None),
                        };

                        return Err(JsExecutionError::new(
                            message,
                            call_stack,
                            raw_assertion_payload,
                        )
                        .into());
                    }
                    ACVMStatus::RequiresForeignCall(foreign_call) => {
                        let result =
                            resolve_brillig(self.foreign_call_handler, &foreign_call).await?;

                        acvm.resolve_pending_foreign_call(result);
                    }
                    ACVMStatus::RequiresAcirCall(call_info) => {
                        let acir_to_call = &self.functions[call_info.id as usize];
                        let initial_witness = call_info.initial_witness;
                        let call_solved_witness = self
                            .execute_circuit(acir_to_call, initial_witness, witness_stack)
                            .await?;
                        let mut call_resolved_outputs = Vec::new();
                        for return_witness_index in acir_to_call.return_values.indices() {
                            if let Some(return_value) =
                                call_solved_witness.get_index(return_witness_index)
                            {
                                call_resolved_outputs.push(*return_value);
                            } else {
                                // TODO: look at changing this call stack from None
                                return Err(JsExecutionError::new(format!("Failed to read from solved witness of ACIR call at witness {}", return_witness_index), None, None).into());
                            }
                        }
                        acvm.resolve_pending_acir_call(call_resolved_outputs);
                        witness_stack.push(call_info.id, call_solved_witness.clone());
                    }
                }
            }

            Ok(acvm.finalize())
        })
    }
}
