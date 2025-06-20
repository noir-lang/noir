use crate::{
    OpcodeResolutionError,
    pwg::{ErrorLocation, input_to_value},
};
use acir::{AcirField, circuit::opcodes::FunctionInput, native_types::WitnessMap};

pub(crate) fn solve_range_opcode<F: AcirField>(
    initial_witness: &WitnessMap<F>,
    input: &FunctionInput<F>,
    pedantic_solving: bool,
) -> Result<(), OpcodeResolutionError<F>> {
    // TODO(https://github.com/noir-lang/noir/issues/5985):
    // re-enable bitsize checks by default
    let skip_bitsize_checks = !pedantic_solving;
    let w_value = input_to_value(initial_witness, *input, skip_bitsize_checks)?;
    if input.num_bits() == 0 {
        // FieldElement requires 1 bit for 0, so it needs special handling.
        if !w_value.is_zero() {
            return Err(OpcodeResolutionError::UnsatisfiedConstrain {
                opcode_location: ErrorLocation::Unresolved,
                payload: None,
            });
        }
    } else if w_value.num_bits() > input.num_bits() {
        return Err(OpcodeResolutionError::UnsatisfiedConstrain {
            opcode_location: ErrorLocation::Unresolved,
            payload: None,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use acir::{
        AcirField, FieldElement,
        circuit::opcodes::FunctionInput,
        native_types::{Witness, WitnessMap},
    };

    use crate::pwg::blackbox::solve_range_opcode;

    #[test]
    fn rejects_too_large_inputs() {
        let witness_map =
            WitnessMap::from(BTreeMap::from([(Witness(0), FieldElement::from(256u32))]));
        let input: FunctionInput<FieldElement> = FunctionInput::witness(Witness(0), 8);
        assert!(solve_range_opcode(&witness_map, &input, false).is_err());
    }

    #[test]
    fn accepts_zero_for_zero_bits() {
        let witness_map = WitnessMap::from(BTreeMap::from([(Witness(0), FieldElement::zero())]));
        let input: FunctionInput<FieldElement> = FunctionInput::witness(Witness(0), 0);
        assert!(solve_range_opcode(&witness_map, &input, false).is_ok());
    }

    #[test]
    fn accepts_valid_inputs() {
        let values: [u32; 4] = [0, 1, 8, 255];

        for value in values {
            let witness_map =
                WitnessMap::from(BTreeMap::from([(Witness(0), FieldElement::from(value))]));
            let input: FunctionInput<FieldElement> = FunctionInput::witness(Witness(0), 8);
            assert!(solve_range_opcode(&witness_map, &input, false).is_ok());
        }
    }
}
