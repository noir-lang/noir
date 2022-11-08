use std::collections::BTreeMap;

use crate::{errors::AbiError, input_parser::InputValue, Abi};
use acvm::FieldElement;

pub fn process_abi_with_verifier_input(
    abi: Abi,
    public_inputs_map: BTreeMap<String, InputValue>,
) -> Result<Vec<FieldElement>, AbiError> {
    // Filter out any private inputs from the ABI.
    let public_abi = abi.public_abi();
    let num_pub_params = public_abi.num_parameters();

    let mut public_inputs = Vec::with_capacity(num_pub_params);

    for (param_name, param_type) in public_abi.parameters.clone().into_iter() {
        let value = public_inputs_map
            .get(&param_name)
            .ok_or_else(|| AbiError::MissingParam(param_name.clone()))?
            .clone();

        if !value.matches_abi(&param_type) {
            return Err(AbiError::TypeMismatch { param_name, param_type, value });
        }

        public_inputs.extend(input_value_into_public_inputs(value, param_name)?);
    }

    // Check that no extra witness values have been provided.
    // Any missing values should be caught by the above for-loop so this only catches extra values.
    if num_pub_params != public_inputs_map.len() {
        let param_names = public_abi.parameter_names();
        let unexpected_params: Vec<String> =
            public_inputs_map
                .keys()
                .filter_map(|param| {
                    if param_names.contains(&param) {
                        None
                    } else {
                        Some(param.to_owned())
                    }
                })
                .collect();
        return Err(AbiError::UnexpectedParams(unexpected_params));
    }

    Ok(public_inputs)
}

fn input_value_into_public_inputs(
    value: InputValue,
    param_name: String,
) -> Result<Vec<FieldElement>, AbiError> {
    let mut public_inputs = Vec::new();
    match value {
        InputValue::Field(elem) => public_inputs.push(elem),
        InputValue::Vec(vec_elem) => public_inputs.extend(vec_elem),
        InputValue::Struct(object) => {
            for (name, value) in object {
                public_inputs.extend(input_value_into_public_inputs(value, name)?)
            }
        }
        InputValue::Undefined => return Err(AbiError::UndefinedInput(param_name)),
    }
    Ok(public_inputs)
}
