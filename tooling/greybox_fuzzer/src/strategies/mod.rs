use acvm::FieldElement;

use field::mutate_field_input_value;
use noirc_abi::{input_parser::InputValue, Abi, AbiType, InputMap, Sign};
use rand::Rng;
use rand_xorshift::XorShiftRng;
use std::{
    collections::{BTreeMap, HashSet},
    iter::zip,
};

mod field;
mod int;
mod uint;

pub(super) fn mutate_input_value(
    abi_type: &AbiType,
    previous_input: &InputValue,
    dictionary: &Vec<FieldElement>,
    prng: &mut XorShiftRng,
) -> InputValue {
    match abi_type {
        // Boolean only has 2 values, there is no point in performing complex logic
        AbiType::Boolean => InputValue::Field(FieldElement::from(prng.gen_range(0u32..=1u32))),
        // Mutate fields in a smart way
        AbiType::Field => mutate_field_input_value(previous_input, dictionary, prng),
        // TODO: IMPLEMENT THESE
        AbiType::Integer { sign, .. } if sign == &Sign::Unsigned => InputValue::Field(4i128.into()),
        AbiType::Integer { .. } => InputValue::Field((2i128 - 4).into()),
        AbiType::String { length } => {
            InputValue::String(String::from_utf8(vec![0x41u8; *length as usize]).unwrap())
        }
        AbiType::Array { length, typ } => {
            let length = *length as usize;
            let input_vector = match previous_input {
                InputValue::Vec(previous_input_vector) => previous_input_vector,
                _ => panic!("Mismatch of AbiType and InputValue should not happen"),
            };
            InputValue::Vec(
                (0..=length)
                    .map(|_x| mutate_input_value(typ, &input_vector[_x], dictionary, prng))
                    .collect(),
            )
        }

        AbiType::Struct { fields, .. } => {
            let input_struct = match previous_input {
                InputValue::Struct(previous_input_struct) => previous_input_struct,
                _ => panic!("Mismatch of AbiType and InputValue should not happen"),
            };
            let fields: Vec<(String, InputValue)> = fields
                .iter()
                .map(|(name, typ)| {
                    (name.clone(), mutate_input_value(typ, &input_struct[name], dictionary, prng))
                })
                .collect();

            let fields: BTreeMap<_, _> = fields.into_iter().collect();
            InputValue::Struct(fields)
        }

        AbiType::Tuple { fields } => {
            let input_vector = match previous_input {
                InputValue::Vec(previous_input_vector) => previous_input_vector,
                _ => panic!("Mismatch of AbiType and InputValue should not happen"),
            };
            let fields: Vec<_> = zip(fields, input_vector)
                .map(|(typ, previous_tuple_input)| {
                    mutate_input_value(typ, previous_tuple_input, dictionary, prng)
                })
                .collect();
            InputValue::Vec(fields)
        }
    }
}

pub(super) fn mutate_input_map(
    abi: &Abi,
    previous_input_map: &InputMap,
    dictionary: &HashSet<FieldElement>,
    prng: &mut XorShiftRng,
) -> InputMap {
    let vec_dictionary: Vec<FieldElement> = dictionary.iter().map(|x| *x).collect();
    let total_params = abi.parameters.len();
    let chosen_param = prng.gen_range(0..total_params);
    abi.parameters
        .iter()
        .enumerate()
        .map(|(idx, param)| {
            (
                param.name.clone(),
                if idx == chosen_param {
                    mutate_input_value(
                        &param.typ,
                        &previous_input_map[&param.name],
                        &vec_dictionary,
                        prng,
                    )
                } else {
                    previous_input_map[&param.name].clone()
                },
            )
        })
        .collect()
}

/// Generate the default input value for a given type
/// false for boolean, 0 for integers and field elements and recursively defined through the first three for others
pub(super) fn generate_default_input_value(abi_type: &AbiType) -> InputValue {
    match abi_type {
        AbiType::Field | AbiType::Integer { .. } | AbiType::Boolean => {
            InputValue::Field(0i128.into())
        }

        AbiType::String { length } => {
            InputValue::String(String::from_utf8(vec![0x0u8; *length as usize]).unwrap())
        }
        AbiType::Array { length, typ } => {
            let length = *length as usize;
            InputValue::Vec((0..=length).map(|_x| generate_default_input_value(typ)).collect())
        }

        AbiType::Struct { fields, .. } => {
            let fields: Vec<(String, InputValue)> = fields
                .iter()
                .map(|(name, typ)| (name.clone(), generate_default_input_value(typ)))
                .collect();

            let fields: BTreeMap<_, _> = fields.into_iter().collect();
            InputValue::Struct(fields)
        }

        AbiType::Tuple { fields } => {
            let fields: Vec<_> =
                fields.iter().map(|typ| generate_default_input_value(typ)).collect();
            InputValue::Vec(fields)
        }
    }
}

/// Generate an input map consisting of default values (0 for field, false for boolean, etc)
/// Used to initialize the corpus for the fuzzer, since the input can't be empty as usually in fuzzers
pub(super) fn generate_default_input_map(abi: &Abi) -> InputMap {
    abi.parameters
        .iter()
        .map(|param| (param.name.clone(), generate_default_input_value(&param.typ)))
        .collect()
}
