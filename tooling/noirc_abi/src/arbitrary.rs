use iter_extended::{btree_map, vecmap};
use prop::collection::vec;
use proptest::prelude::*;

use acvm::{AcirField, FieldElement};

use crate::{
    input_parser::InputValue, Abi, AbiParameter, AbiReturnType, AbiType, AbiVisibility, InputMap,
    Sign,
};
use std::collections::{BTreeMap, HashSet};

pub(super) use proptest_derive::Arbitrary;

/// Mutates an iterator of mutable references to [`String`]s to ensure that all values are unique.
fn ensure_unique_strings<'a>(iter: impl Iterator<Item = &'a mut String>) {
    let mut seen_values: HashSet<String> = HashSet::default();
    for value in iter {
        while seen_values.contains(value.as_str()) {
            value.push('1');
        }
        seen_values.insert(value.clone());
    }
}

proptest::prop_compose! {
    pub(super) fn arb_field_from_integer(bit_size: u32)(value: u128)-> FieldElement {
        let width = (bit_size % 128).clamp(1, 127);
        let max_value = 2u128.pow(width) - 1;
        FieldElement::from(value.clamp(0, max_value))
    }
}

fn arb_value_from_abi_type(abi_type: &AbiType) -> SBoxedStrategy<InputValue> {
    match abi_type {
        AbiType::Field => vec(any::<u8>(), 32)
            .prop_map(|bytes| InputValue::Field(FieldElement::from_be_bytes_reduce(&bytes)))
            .sboxed(),
        AbiType::Integer { width, .. } => {
            arb_field_from_integer(*width).prop_map(InputValue::Field).sboxed()
        }

        AbiType::Boolean => {
            any::<bool>().prop_map(|val| InputValue::Field(FieldElement::from(val))).sboxed()
        }

        AbiType::String { length } => {
            // Strings only allow ASCII characters as each character must be able to be represented by a single byte.
            let string_regex = format!("[[:ascii:]]{{{length}}}");
            proptest::string::string_regex(&string_regex)
                .expect("parsing of regex should always succeed")
                .prop_map(InputValue::String)
                .sboxed()
        }
        AbiType::Array { length, typ } => {
            let length = *length as usize;
            let elements = vec(arb_value_from_abi_type(typ), length..=length);

            elements.prop_map(InputValue::Vec).sboxed()
        }

        AbiType::Struct { fields, .. } => {
            let fields: Vec<SBoxedStrategy<(String, InputValue)>> = fields
                .iter()
                .map(|(name, typ)| (Just(name.clone()), arb_value_from_abi_type(typ)).sboxed())
                .collect();

            fields
                .prop_map(|fields| {
                    let fields: BTreeMap<_, _> = fields.into_iter().collect();
                    InputValue::Struct(fields)
                })
                .sboxed()
        }

        AbiType::Tuple { fields } => {
            let fields: Vec<_> = fields.iter().map(arb_value_from_abi_type).collect();
            fields.prop_map(InputValue::Vec).sboxed()
        }
    }
}

fn arb_primitive_abi_type() -> SBoxedStrategy<AbiType> {
    const MAX_STRING_LEN: u32 = 1000;
    proptest::prop_oneof![
        Just(AbiType::Field),
        Just(AbiType::Boolean),
        any::<(Sign, u32)>().prop_map(|(sign, width)| {
            let width = (width % 128).clamp(1, 127);
            AbiType::Integer { sign, width }
        }),
        // restrict length of strings to avoid running out of memory
        (1..MAX_STRING_LEN).prop_map(|length| AbiType::String { length }),
    ]
    .sboxed()
}

pub(super) fn arb_abi_type() -> BoxedStrategy<AbiType> {
    let leaf = arb_primitive_abi_type();

    leaf.prop_recursive(
        8,   // up to 8 levels deep
        256, // Shoot for maximum size of 256 nodes
        10,  // We put up to 10 items per collection
        |inner| {
            prop_oneof![
                (1..10u32, inner.clone())
                    .prop_map(|(length, typ)| { AbiType::Array { length, typ: Box::new(typ) } })
                    .boxed(),
                vec(inner.clone(), 1..10).prop_map(|fields| { AbiType::Tuple { fields } }).boxed(),
                (".*", vec((".+", inner), 1..10))
                    .prop_map(|(path, mut fields)| {
                        // Require that all field names are unique.
                        ensure_unique_strings(fields.iter_mut().map(|(field_name, _)| field_name));
                        AbiType::Struct { path, fields }
                    })
                    .boxed(),
            ]
        },
    )
    .boxed()
}

fn arb_abi_param_and_value() -> BoxedStrategy<(AbiParameter, InputValue)> {
    arb_abi_type()
        .prop_flat_map(|typ| {
            let value = arb_value_from_abi_type(&typ);
            let param = arb_abi_param(typ);
            (param, value)
        })
        .boxed()
}

fn arb_abi_param(typ: AbiType) -> SBoxedStrategy<AbiParameter> {
    (".+", any::<AbiVisibility>())
        .prop_map(move |(name, visibility)| AbiParameter { name, typ: typ.clone(), visibility })
        .sboxed()
}

prop_compose! {
    pub(super) fn arb_abi_and_input_map()
        (mut parameters_with_values in vec(arb_abi_param_and_value(), 0..100), return_type: Option<AbiReturnType>)
        -> (Abi, InputMap) {
            // Require that all parameter names are unique.
            ensure_unique_strings(parameters_with_values.iter_mut().map(|(param_name,_)| &mut param_name.name));

            let parameters  = vecmap(&parameters_with_values, |(param, _)| param.clone());
            let input_map = btree_map(parameters_with_values, |(param, value)| (param.name, value));

            (Abi { parameters, return_type, error_types: BTreeMap::default() }, input_map)
    }
}
