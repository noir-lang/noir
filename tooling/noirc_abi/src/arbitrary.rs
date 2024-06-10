use iter_extended::{btree_map, vecmap};
use proptest::prelude::*;

use acvm::FieldElement;

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

fn arb_primitive_abi_type_and_value(
) -> impl proptest::strategy::Strategy<Value = (AbiType, InputValue)> {
    proptest::prop_oneof![
        any::<u128>().prop_map(|val| (AbiType::Field, InputValue::Field(FieldElement::from(val)))),
        any::<(Sign, u32)>().prop_flat_map(|(sign, width)| {
            let width = (width % 128).clamp(1, 127);
            (
                Just(AbiType::Integer { sign, width }),
                arb_field_from_integer(width).prop_map(InputValue::Field),
            )
        }),
        any::<bool>()
            .prop_map(|val| (AbiType::Boolean, InputValue::Field(FieldElement::from(val)))),
        ".+".prop_map(|str| (
            AbiType::String { length: str.len() as u32 },
            InputValue::String(str)
        ))
    ]
}

fn arb_abi_type_and_value() -> impl proptest::strategy::Strategy<Value = (AbiType, InputValue)> {
    let leaf = arb_primitive_abi_type_and_value();

    leaf.prop_recursive(
        8,   // 8 levels deep
        256, // Shoot for maximum size of 256 nodes
        10,  // We put up to 10 items per collection
        |inner| {
            prop_oneof![
                // TODO: support `AbiType::Array`.
                // This is non-trivial due to the need to get N `InputValue`s which are all compatible with
                // the element's `AbiType`.`
                prop::collection::vec(inner.clone(), 1..10).prop_map(|typ| {
                    let (fields, values): (Vec<_>, Vec<_>) = typ.into_iter().unzip();
                    let tuple_type = AbiType::Tuple { fields };
                    (tuple_type, InputValue::Vec(values))
                }),
                (".*", prop::collection::vec((inner.clone(), ".*"), 1..10)).prop_map(
                    |(path, mut typ)| {
                        // Require that all field names are unique.
                        ensure_unique_strings(typ.iter_mut().map(|(_, field_name)| field_name));

                        let (types_and_values, names): (Vec<_>, Vec<_>) = typ.into_iter().unzip();
                        let (types, values): (Vec<_>, Vec<_>) =
                            types_and_values.into_iter().unzip();

                        let fields = names.clone().into_iter().zip(types).collect();
                        let struct_values = names.into_iter().zip(values).collect();
                        let struct_type = AbiType::Struct { path, fields };

                        (struct_type, InputValue::Struct(struct_values))
                    }
                ),
            ]
        },
    )
}

proptest::prop_compose! {
    pub(super) fn arb_abi_type()((typ, _) in arb_abi_type_and_value())-> AbiType {
        typ
    }
}

prop_compose! {
    fn arb_abi_param_and_value()
                ((typ, value) in arb_abi_type_and_value(), name: String, visibility: AbiVisibility)
                -> (AbiParameter, InputValue) {
        (AbiParameter{ name, typ, visibility }, value)
    }
}

prop_compose! {
    pub(super) fn arb_abi_and_input_map()
        (mut parameters_with_values in proptest::collection::vec(arb_abi_param_and_value(), 0..100), return_type: Option<AbiReturnType>)
        -> (Abi, InputMap) {
            // Require that all parameter names are unique.
            ensure_unique_strings(parameters_with_values.iter_mut().map(|(param_name,_)| &mut param_name.name));

            let parameters  = vecmap(&parameters_with_values, |(param, _)| param.clone());
            let input_map = btree_map(parameters_with_values, |(param, value)| (param.name, value));

            (Abi { parameters, return_type, error_types: BTreeMap::default() }, input_map)
    }
}
