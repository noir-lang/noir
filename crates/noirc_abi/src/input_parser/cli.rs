use crate::{errors::InputParserError, AbiType};

use super::{parse_str_to_field, InputValue};

impl InputValue {
    pub fn try_from_cli_args(
        value: String,
        param_type: &AbiType,
    ) -> Result<InputValue, InputParserError> {
        let input_value = match param_type {
            AbiType::String { .. } => InputValue::String(value),
            AbiType::Field | AbiType::Integer { .. } => {
                InputValue::Field(parse_str_to_field(&value)?)
            }
            _ => return Err(InputParserError::AbiTypeMismatch(param_type.clone())),
        };
        // TODO: support complex types
        Ok(input_value)
    }
}
