use crate::function_context::{FieldRepresentation, WitnessValue};
use crate::{NUMBER_OF_PREDEFINED_VARIABLES, NUMBER_OF_VARIABLES_INITIAL};
use noir_ssa_fuzzer::r#type::{NumericType, Type};

/// Creates default witness values for testing
/// Returns [Field(0), Field(1), Field(2), Field(3), Field(4)]
pub(crate) fn default_witness()
-> [WitnessValue; (NUMBER_OF_VARIABLES_INITIAL - NUMBER_OF_PREDEFINED_VARIABLES) as usize] {
    [
        WitnessValue::Field(FieldRepresentation { high: 0, low: 0 }),
        WitnessValue::Field(FieldRepresentation { high: 0, low: 1 }),
        WitnessValue::Field(FieldRepresentation { high: 0, low: 2 }),
        WitnessValue::Field(FieldRepresentation { high: 0, low: 3 }),
        WitnessValue::Field(FieldRepresentation { high: 0, low: 4 }),
    ]
}

pub(crate) fn default_input_types() -> Vec<Type> {
    vec![
        Type::Numeric(NumericType::Field);
        (NUMBER_OF_VARIABLES_INITIAL - NUMBER_OF_PREDEFINED_VARIABLES) as usize
    ]
}
