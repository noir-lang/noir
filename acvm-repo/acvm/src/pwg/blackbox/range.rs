use crate::{
    OpcodeResolutionError,
    pwg::{ErrorLocation, input_to_value},
};
use acir::{AcirField, circuit::opcodes::FunctionInput, native_types::WitnessMap};

pub(crate) fn solve_range_opcode<F: AcirField>(
    initial_witness: &WitnessMap<F>,
    input: &FunctionInput<F>,
    num_bits: u32,
) -> Result<(), OpcodeResolutionError<F>> {
    let w_value = input_to_value(initial_witness, *input)?;

    if w_value.num_bits() > num_bits {
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
        let input: FunctionInput<FieldElement> = FunctionInput::Witness(Witness(0));
        assert!(solve_range_opcode(&witness_map, &input, 8).is_err());
    }

    #[test]
    fn accepts_zero_for_zero_bits() {
        let witness_map = WitnessMap::from(BTreeMap::from([(Witness(0), FieldElement::zero())]));
        let input: FunctionInput<FieldElement> = FunctionInput::Witness(Witness(0));
        assert!(solve_range_opcode(&witness_map, &input, 0).is_ok());
    }

    #[test]
    fn accepts_valid_inputs() {
        let values: [u32; 4] = [0, 1, 8, 255];

        for value in values {
            let witness_map =
                WitnessMap::from(BTreeMap::from([(Witness(0), FieldElement::from(value))]));
            let input: FunctionInput<FieldElement> = FunctionInput::Witness(Witness(0));
            assert!(solve_range_opcode(&witness_map, &input, 8).is_ok());
        }
    }
}
