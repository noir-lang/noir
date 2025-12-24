use acir::{AcirField, circuit::opcodes::FunctionInput, native_types::WitnessMap};

use crate::pwg::{OpcodeResolutionError, input_to_value};

pub(crate) fn to_u8_array<const N: usize, F: AcirField>(
    initial_witness: &WitnessMap<F>,
    inputs: &[FunctionInput<F>; N],
) -> Result<[u8; N], OpcodeResolutionError<F>> {
    let mut result = [0; N];
    for (it, input) in result.iter_mut().zip(inputs) {
        let byte: u8 = input_to_value(initial_witness, *input)?
            .try_into_u128()
            .expect("expected input to fit into a u8")
            .try_into()
            .expect("expected input to fit into a u8");
        *it = byte;
    }
    Ok(result)
}

pub(crate) fn to_u8_vec<F: AcirField>(
    initial_witness: &WitnessMap<F>,
    inputs: &[FunctionInput<F>],
) -> Result<Vec<u8>, OpcodeResolutionError<F>> {
    let mut result = Vec::with_capacity(inputs.len());
    for input in inputs {
        let byte: u8 = input_to_value(initial_witness, *input)?
            .try_into_u128()
            .expect("expected input to fit into a u8")
            .try_into()
            .expect("expected input to fit into a u8");
        result.push(byte);
    }
    Ok(result)
}
