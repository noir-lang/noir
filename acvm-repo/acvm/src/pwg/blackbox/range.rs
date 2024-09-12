use crate::{
    pwg::{input_to_value, ErrorLocation},
    OpcodeResolutionError,
};
use acir::{circuit::opcodes::FunctionInput, native_types::WitnessMap, AcirField};

pub(crate) fn solve_range_opcode<F: AcirField>(
    initial_witness: &WitnessMap<F>,
    input: &FunctionInput<F>,
) -> Result<(), OpcodeResolutionError<F>> {
    // TODO(https://github.com/noir-lang/noir/issues/5985):
    // re-enable bitsize checks
    let skip_bitsize_checks = true;
    let w_value = input_to_value(initial_witness, *input, skip_bitsize_checks)?;
    if w_value.num_bits() > input.num_bits() {
        return Err(OpcodeResolutionError::UnsatisfiedConstrain {
            opcode_location: ErrorLocation::Unresolved,
            payload: None,
        });
    }
    Ok(())
}
