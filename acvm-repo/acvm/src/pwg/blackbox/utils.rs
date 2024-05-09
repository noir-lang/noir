use acir::{circuit::opcodes::FunctionInput, native_types::WitnessMap};

use crate::pwg::{witness_to_value, OpcodeResolutionError};

pub(crate) fn to_u8_array<const N: usize>(
    initial_witness: &WitnessMap,
    inputs: &[FunctionInput; N],
) -> Result<[u8; N], OpcodeResolutionError> {
    let mut result = [0; N];
    for (it, input) in result.iter_mut().zip(inputs) {
        let witness_value_bytes = witness_to_value(initial_witness, input.witness)?.to_be_bytes();
        let byte = witness_value_bytes
            .last()
            .expect("Field element must be represented by non-zero amount of bytes");
        *it = *byte;
    }
    Ok(result)
}

pub(crate) fn to_u8_vec(
    initial_witness: &WitnessMap,
    inputs: &[FunctionInput],
) -> Result<Vec<u8>, OpcodeResolutionError> {
    let mut result = Vec::with_capacity(inputs.len());
    for input in inputs {
        let witness_value_bytes = witness_to_value(initial_witness, input.witness)?.to_be_bytes();
        let byte = witness_value_bytes
            .last()
            .expect("Field element must be represented by non-zero amount of bytes");
        result.push(*byte);
    }
    Ok(result)
}
