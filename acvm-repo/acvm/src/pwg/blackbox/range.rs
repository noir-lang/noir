use crate::{
    pwg::{input_to_value, ErrorLocation},
    OpcodeResolutionError,
};
use acir::{circuit::opcodes::FunctionInput, native_types::WitnessMap, AcirField};

pub(crate) fn solve_range_opcode<F: AcirField>(
    initial_witness: &WitnessMap<F>,
    input: &FunctionInput<F>,
) -> Result<(), OpcodeResolutionError<F>> {
    let w_value = input_to_value(initial_witness, *input)?;
    if w_value.num_bits() > input.num_bits() {
        return Err(OpcodeResolutionError::UnsatisfiedConstrain {
            opcode_location: ErrorLocation::Unresolved,
            payload: None,
        });
    }
    Ok(())
}
