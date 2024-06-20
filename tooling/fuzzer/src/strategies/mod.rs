use int::IntStrategy;
use prop::collection::vec;
use proptest::prelude::*;

use acvm::{AcirField, FieldElement};

use noirc_abi::{input_parser::InputValue, Abi, AbiType, InputMap, Sign};
use std::collections::BTreeMap;
use uint::UintStrategy;

mod int;
mod uint;

proptest::prop_compose! {
    pub(super) fn arb_field_from_integer(bit_size: u32)(value: u128)-> FieldElement {
        let width = (bit_size % 128).clamp(1, 127);
        let max_value = 2u128.pow(width) - 1;
        let value = value % max_value;
        FieldElement::from(value)
    }
}

pub(super) fn arb_value_from_abi_type(abi_type: &AbiType) -> SBoxedStrategy<InputValue> {
    match abi_type {
        AbiType::Field => vec(any::<u8>(), 32)
            .prop_map(|bytes| InputValue::Field(FieldElement::from_be_bytes_reduce(&bytes)))
            .sboxed(),
        AbiType::Integer { width, sign } if sign == &Sign::Unsigned => {
            UintStrategy::new(*width as usize)
                .prop_map(|uint| InputValue::Field(uint.into()))
                .sboxed()
        }
        AbiType::Integer { width, .. } => {
            let shift = 2i128.pow(*width);
            IntStrategy::new(*width as usize)
                .prop_map(move |mut int| {
                    if int < 0 {
                        int += shift
                    }
                    InputValue::Field(int.into())
                })
                .sboxed()
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

pub(super) fn arb_input_map(abi: &Abi) -> BoxedStrategy<InputMap> {
    let values: Vec<_> = abi
        .parameters
        .iter()
        .map(|param| (Just(param.name.clone()), arb_value_from_abi_type(&param.typ)))
        .collect();

    values
        .prop_map(|values| {
            let input_map: InputMap = values.into_iter().collect();
            input_map
        })
        .boxed()
}
