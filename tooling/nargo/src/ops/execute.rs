use alloy_primitives::U256;
use noirc_abi::InputMap;
use crate::NargoError;
use crate::errors::ExecutionError;
use crate::foreign_calls::ForeignCallExecutor;

/// Stub implementation - execution requires ACVM backend
pub fn execute_program<E: ForeignCallExecutor<U256>>(
    _program: &Vec<u8>, // Placeholder for compiled program
    _initial_witness: InputMap,
    _foreign_call_executor: &mut E,
) -> Result<Vec<U256>, NargoError> {
    Err(NargoError::ExecutionError(ExecutionError::General(
        "Program execution is not available in Sensei (requires ZK backend)".to_string()
    )))
}

/// Stub for execution with fuzzing
pub fn execute_program_with_acir_fuzzing<E: ForeignCallExecutor<U256>>(
    _program: &Vec<u8>,
    _initial_witness: InputMap,
    _foreign_call_executor: &mut E,
) -> Result<(Vec<U256>, Option<Vec<u32>>), (NargoError, Option<Vec<u32>>)> {
    Err((
        NargoError::ExecutionError(ExecutionError::General(
            "Fuzzing execution is not available in Sensei (requires ZK backend)".to_string()
        )),
        None
    ))
}

/// Stub for Brillig execution with fuzzing
pub fn execute_program_with_brillig_fuzzing<E: ForeignCallExecutor<U256>>(
    _program: &Vec<u8>,
    _initial_witness: InputMap,
    _foreign_call_executor: &mut E,
) -> Result<(Vec<U256>, Vec<(usize, Vec<u32>)>), (NargoError, Vec<(usize, Vec<u32>)>)> {
    Err((
        NargoError::ExecutionError(ExecutionError::General(
            "Brillig fuzzing execution is not available in Sensei (requires ZK backend)".to_string()
        )),
        Vec::new()
    ))
}