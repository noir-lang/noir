use crate::{
    pwg::{witness_to_value, ErrorLocation},
    OpcodeResolutionError,
};
use acir::{circuit::opcodes::FunctionInput, native_types::WitnessMap};

pub(super) fn solve_range_opcode(
    initial_witness: &WitnessMap,
    input: &FunctionInput,
) -> Result<(), OpcodeResolutionError> {
    let w_value = witness_to_value(initial_witness, input.witness)?;
    if w_value.num_bits() > input.num_bits {
        return Err(OpcodeResolutionError::UnsatisfiedConstrain {
            opcode_location: ErrorLocation::Unresolved,
        });
    }
    Ok(())
}
