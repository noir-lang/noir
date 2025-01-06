use std::collections::BTreeMap;

use acvm::acir::AcirField;
use iter_extended::vecmap;

use noirc_printable_type::{PrintableType, PrintableValue};

use crate::decode_string_value;

/// Assumes that `field_iterator` contains enough field elements in order to decode the [PrintableType]
pub fn decode_value<F: AcirField>(
    field_iterator: &mut impl Iterator<Item = F>,
    typ: &PrintableType,
) -> PrintableValue<F> {
    match typ {
        PrintableType::Field
        | PrintableType::SignedInteger { .. }
        | PrintableType::UnsignedInteger { .. }
        | PrintableType::Boolean => {
            let field_element = field_iterator.next().unwrap();

            PrintableValue::Field(field_element)
        }
        PrintableType::Array { length, typ } => {
            let length = *length as usize;
            let mut array_elements = Vec::with_capacity(length);
            for _ in 0..length {
                array_elements.push(decode_value(field_iterator, typ));
            }

            PrintableValue::Vec { array_elements, is_slice: false }
        }
        PrintableType::Slice { typ } => {
            let length = field_iterator
                .next()
                .expect("not enough data to decode variable array length")
                .to_u128() as usize;
            let mut array_elements = Vec::with_capacity(length);
            for _ in 0..length {
                array_elements.push(decode_value(field_iterator, typ));
            }

            PrintableValue::Vec { array_elements, is_slice: true }
        }
        PrintableType::Tuple { types } => PrintableValue::Vec {
            array_elements: vecmap(types, |typ| decode_value(field_iterator, typ)),
            is_slice: false,
        },
        PrintableType::String { length } => {
            let field_elements: Vec<F> = field_iterator.take(*length as usize).collect();

            PrintableValue::String(decode_string_value(&field_elements))
        }
        PrintableType::Struct { fields, .. } => {
            let mut struct_map = BTreeMap::new();

            for (field_key, param_type) in fields {
                let field_value = decode_value(field_iterator, param_type);

                struct_map.insert(field_key.to_owned(), field_value);
            }

            PrintableValue::Struct(struct_map)
        }
        PrintableType::Function { env, .. } => {
            let field_element = field_iterator.next().unwrap();
            let func_ref = PrintableValue::Field(field_element);
            // we want to consume the fields from the environment, but for now they are not actually printed
            decode_value(field_iterator, env);
            func_ref
        }
        PrintableType::MutableReference { typ } => {
            // we decode the reference, but it's not really used for printing
            decode_value(field_iterator, typ)
        }
        PrintableType::Unit => PrintableValue::Field(F::zero()),
    }
}
