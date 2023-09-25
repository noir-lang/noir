use acir::{circuit::opcodes::FunctionInput, native_types::WitnessMap};

use crate::pwg::{witness_to_value, OpcodeResolutionError};

fn to_u8_vec(
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

pub(super) mod ecdsa;
pub(super) mod schnorr;
