use std::collections::BTreeMap;

use crate::errors::AbiError;
use crate::AbiType;
use crate::{input_parser::InputValue, Abi};
use acvm::acir::native_types::Witness;
use acvm::FieldElement;

use super::MAIN_RETURN_NAME;

/// In Barretenberg, the proof system adds a zero witness in the first index,
/// So when we add witness values, their index start from 1.
const WITNESS_OFFSET: u32 = 1;

/// Ordering is important here, which is why we need the ABI to tell us what order to add the elements in
/// We then need the witness map to get the elements field values.
pub fn process_abi_with_input(
    abi: Abi,
    witness_map: &BTreeMap<String, InputValue>,
) -> Result<(BTreeMap<Witness, FieldElement>, Option<Witness>), AbiError> {
    let num_params = abi.num_parameters();
    let mut solved_witness = BTreeMap::new();

    let mut index = 0;
    let mut return_witness = None;
    let return_witness_len =
        if let Some(return_param) = abi.parameters.iter().find(|x| x.0 == MAIN_RETURN_NAME) {
            match &return_param.1 {
                AbiType::Array { length, .. } => *length as u32,
                AbiType::Integer { .. } | AbiType::Field(_) => 1,
                AbiType::Struct { fields, .. } => fields.len() as u32,
            }
        } else {
            0
        };
    for (param_name, param_type) in abi.parameters.clone().into_iter() {
        let value = witness_map
            .get(&param_name)
            .ok_or_else(|| AbiError::MissingParam(param_name.clone()))?
            .clone();

        if !value.matches_abi(&param_type) {
            return Err(AbiError::TypeMismatch { param_name, param_type, value });
        }

        (index, return_witness) = input_value_into_witness(
            value,
            index,
            return_witness,
            &mut solved_witness,
            param_name,
            return_witness_len,
        )?;
    }

    // Check that no extra witness values have been provided.
    // Any missing values should be caught by the above for-loop so this only catches extra values.
    if num_params != witness_map.len() {
        let param_names = abi.parameter_names();
        let unexpected_params: Vec<String> =
            witness_map
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

    Ok((solved_witness, return_witness))
}

fn input_value_into_witness(
    value: InputValue,
    initial_index: u32,
    initial_return_witness: Option<Witness>,
    solved_witness: &mut BTreeMap<Witness, FieldElement>,
    param_name: String,
    return_witness_len: u32,
) -> Result<(u32, Option<Witness>), AbiError> {
    let mut index = initial_index;
    let mut return_witness = initial_return_witness;
    match value {
        InputValue::Field(element) => {
            let old_value = solved_witness.insert(Witness::new(index + WITNESS_OFFSET), element);
            assert!(old_value.is_none());
            index += 1;
        }
        InputValue::Vec(arr) => {
            for element in arr {
                let old_value =
                    solved_witness.insert(Witness::new(index + WITNESS_OFFSET), element);
                assert!(old_value.is_none());
                index += 1;
            }
        }
        InputValue::Struct(object) => {
            for (name, value) in object {
                (index, return_witness) = input_value_into_witness(
                    value,
                    index,
                    return_witness,
                    solved_witness,
                    name,
                    return_witness_len,
                )?;
            }
        }
        InputValue::Undefined => {
            if param_name != MAIN_RETURN_NAME {
                return Err(AbiError::UndefinedInput(param_name));
            }

            return_witness = Some(Witness::new(index + WITNESS_OFFSET));

            //We do not support undefined arrays for now - TODO
            if return_witness_len != 1 {
                return Err(AbiError::Generic(
                    "Values of array returned from main must be specified".to_string(),
                ));
            }
            index += return_witness_len;
            //XXX We do not support (yet) array of arrays
        }
    }

    Ok((index, return_witness))
}
