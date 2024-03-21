use acvm::{
    acir::circuit::Program,
    pwg::{ACVMStatus, ErrorLocation, OpcodeResolutionError, ACVM},
};
use bn254_blackbox_solver::Bn254BlackBoxSolver;

use js_sys::Error;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    foreign_call::{resolve_brillig, ForeignCallHandler},
    JsExecutionError, JsWitnessMap,
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

    execute_circuit_with_black_box_solver(&solver, program, initial_witness, foreign_call_handler)
        .await
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
    // TODO(https://github.com/noir-lang/noir/issues/4428): These need to be updated to match the same interfaces
    // as the native ACVM executor. Right now native execution still only handles one circuit so I do not feel the need
    // to break the JS interface just yet.
    program: Vec<u8>,
    initial_witness: JsWitnessMap,
    foreign_call_handler: ForeignCallHandler,
) -> Result<JsWitnessMap, Error> {
    console_error_panic_hook::set_once();
    let program: Program = Program::deserialize_program(&program)
        .map_err(|_| JsExecutionError::new("Failed to deserialize circuit. This is likely due to differing serialization formats between ACVM_JS and your compiler".to_string(), None))?;
    let circuit = match program.functions.len() {
        0 => return Ok(initial_witness),
        1 => &program.functions[0],
        _ => return Err(JsExecutionError::new("Program contains multiple circuits however ACVM currently only supports programs containing a single circuit".to_string(), None).into())
    };

    let mut acvm = ACVM::new(&solver.0, &circuit.opcodes, initial_witness.into());

    loop {
        let solver_status = acvm.solve();

        match solver_status {
            ACVMStatus::Solved => break,
            ACVMStatus::InProgress => {
                unreachable!("Execution should not stop while in `InProgress` state.")
            }
            ACVMStatus::Failure(error) => {
                let (assert_message, call_stack) = match &error {
                    OpcodeResolutionError::UnsatisfiedConstrain {
                        opcode_location: ErrorLocation::Resolved(opcode_location),
                    }
                    | OpcodeResolutionError::IndexOutOfBounds {
                        opcode_location: ErrorLocation::Resolved(opcode_location),
                        ..
                    } => {
                        (circuit.get_assert_message(*opcode_location), Some(vec![*opcode_location]))
                    }
                    OpcodeResolutionError::BrilligFunctionFailed { call_stack, .. } => {
                        let failing_opcode =
                            call_stack.last().expect("Brillig error call stacks cannot be empty");
                        (circuit.get_assert_message(*failing_opcode), Some(call_stack.clone()))
                    }
                    _ => (None, None),
                };

                let error_string = match &assert_message {
                    Some(assert_message) => format!("Assertion failed: {}", assert_message),
                    None => error.to_string(),
                };

                return Err(JsExecutionError::new(error_string.into(), call_stack).into());
            }
            ACVMStatus::RequiresForeignCall(foreign_call) => {
                let result = resolve_brillig(&foreign_call_handler, &foreign_call).await?;

                acvm.resolve_pending_foreign_call(result);
            }
        }
    }

    let witness_map = acvm.finalize();
    Ok(witness_map.into())
}
