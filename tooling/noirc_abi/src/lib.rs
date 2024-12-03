#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

use acvm::{
    acir::{
        circuit::ErrorSelector,
        native_types::{Witness, WitnessMap},
    },
    AcirField, FieldElement,
};
use errors::AbiError;
use input_parser::InputValue;
use iter_extended::{try_btree_map, try_vecmap, vecmap};
use noirc_printable_type::{
    decode_value as printable_type_decode_value, PrintableType, PrintableValue,
    PrintableValueDisplay,
};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::{collections::BTreeMap, str};
// This is the ABI used to bridge the different TOML formats for the initial
// witness, the partial witness generator and the interpreter.
//
// This ABI has nothing to do with ACVM or ACIR. Although they implicitly have a relationship

#[cfg(test)]
mod arbitrary;

pub mod errors;
pub mod input_parser;
mod serialization;

/// A map from the fields in an TOML/JSON file which correspond to some ABI to their values
pub type InputMap = BTreeMap<String, InputValue>;

pub const MAIN_RETURN_NAME: &str = "return";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
/// Types that are allowed in the (main function in binary)
///
/// we use this separation so that we can have types like Strings
/// without needing to introduce this in the Noir types
///
/// NOTE: If Strings are introduced as a native type, the translation will
/// be straightforward. Whether exotic types like String will be natively supported
/// depends on the types of programs that users want to do. I don't envision string manipulation
/// in programs, however it is possible to support, with many complications like encoding character set
/// support.
#[derive(Hash)]
pub enum AbiType {
    Field,
    Array {
        length: u32,
        #[serde(rename = "type")]
        typ: Box<AbiType>,
    },
    Integer {
        sign: Sign,
        width: u32,
    },
    Boolean,
    Struct {
        path: String,
        #[serde(
            serialize_with = "serialization::serialize_struct_fields",
            deserialize_with = "serialization::deserialize_struct_fields"
        )]
        fields: Vec<(String, AbiType)>,
    },
    Tuple {
        fields: Vec<AbiType>,
    },
    String {
        length: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
#[serde(rename_all = "lowercase")]
/// Represents whether the parameter is public or known only to the prover.
pub enum AbiVisibility {
    Public,
    // Constants are not allowed in the ABI for main at the moment.
    // Constant,
    Private,
    DataBus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
#[serde(rename_all = "lowercase")]
pub enum Sign {
    Unsigned,
    Signed,
}

impl AbiType {
    /// Returns the number of field elements required to represent the type once encoded.
    pub fn field_count(&self) -> u32 {
        match self {
            AbiType::Field | AbiType::Integer { .. } | AbiType::Boolean => 1,
            AbiType::Array { length, typ } => typ.field_count() * *length,
            AbiType::Struct { fields, .. } => {
                fields.iter().fold(0, |acc, (_, field_type)| acc + field_type.field_count())
            }
            AbiType::Tuple { fields } => {
                fields.iter().fold(0, |acc, field_typ| acc + field_typ.field_count())
            }
            AbiType::String { length } => *length,
        }
    }
}

impl From<&AbiType> for PrintableType {
    fn from(value: &AbiType) -> Self {
        match value {
            AbiType::Field => PrintableType::Field,
            AbiType::String { length } => PrintableType::String { length: *length },
            AbiType::Tuple { fields } => {
                let fields = fields.iter().map(|field| field.into()).collect();
                PrintableType::Tuple { types: fields }
            }
            AbiType::Array { length, typ } => {
                let borrowed: &AbiType = typ.borrow();
                PrintableType::Array { length: *length, typ: Box::new(borrowed.into()) }
            }
            AbiType::Boolean => PrintableType::Boolean,
            AbiType::Struct { path, fields } => {
                let fields =
                    fields.iter().map(|(name, field)| (name.clone(), field.into())).collect();
                PrintableType::Struct {
                    name: path.split("::").last().unwrap_or_default().to_string(),
                    fields,
                }
            }
            AbiType::Integer { sign: Sign::Unsigned, width } => {
                PrintableType::UnsignedInteger { width: *width }
            }
            AbiType::Integer { sign: Sign::Signed, width } => {
                PrintableType::SignedInteger { width: *width }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
/// An argument or return value of the circuit's `main` function.
pub struct AbiParameter {
    pub name: String,
    #[serde(rename = "type")]
    #[cfg_attr(test, proptest(strategy = "arbitrary::arb_abi_type()"))]
    pub typ: AbiType,
    pub visibility: AbiVisibility,
}

impl AbiParameter {
    pub fn is_public(&self) -> bool {
        self.visibility == AbiVisibility::Public
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub struct AbiReturnType {
    #[cfg_attr(test, proptest(strategy = "arbitrary::arb_abi_type()"))]
    pub abi_type: AbiType,
    pub visibility: AbiVisibility,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub struct Abi {
    /// An ordered list of the arguments to the program's `main` function, specifying their types and visibility.
    pub parameters: Vec<AbiParameter>,
    pub return_type: Option<AbiReturnType>,
    #[cfg_attr(test, proptest(strategy = "proptest::prelude::Just(BTreeMap::from([]))"))]
    pub error_types: BTreeMap<ErrorSelector, AbiErrorType>,
}

impl Abi {
    pub fn parameter_names(&self) -> Vec<&String> {
        self.parameters.iter().map(|x| &x.name).collect()
    }

    pub fn num_parameters(&self) -> usize {
        self.parameters.len()
    }

    /// Returns the number of field elements required to represent the ABI's input once encoded.
    pub fn field_count(&self) -> u32 {
        self.parameters.iter().map(|param| param.typ.field_count()).sum()
    }

    /// Returns whether any values are needed to be made public for verification.
    pub fn has_public_inputs(&self) -> bool {
        let has_public_args = self.parameters.iter().any(|param| param.is_public());
        let has_public_return = self
            .return_type
            .as_ref()
            .map_or(false, |typ| matches!(typ.visibility, AbiVisibility::Public));
        has_public_args || has_public_return
    }

    /// Returns `true` if the ABI contains no parameters or return value.
    pub fn is_empty(&self) -> bool {
        self.return_type.is_none() && self.parameters.is_empty()
    }

    pub fn to_btree_map(&self) -> BTreeMap<String, AbiType> {
        let mut map = BTreeMap::new();
        for param in self.parameters.iter() {
            map.insert(param.name.clone(), param.typ.clone());
        }
        map
    }

    /// Encode a set of inputs as described in the ABI into a `WitnessMap`.
    pub fn encode(
        &self,
        input_map: &InputMap,
        return_value: Option<InputValue>,
    ) -> Result<WitnessMap<FieldElement>, AbiError> {
        // Check that no extra witness values have been provided.
        let param_names = self.parameter_names();
        if param_names.len() < input_map.len() {
            let unexpected_params: Vec<String> =
                input_map.keys().filter(|param| !param_names.contains(param)).cloned().collect();
            return Err(AbiError::UnexpectedParams(unexpected_params));
        }

        // First encode each input separately, performing any input validation.
        let mut encoded_inputs: Vec<Vec<FieldElement>> = self
            .parameters
            .iter()
            .map(|param| {
                let value = input_map
                    .get(&param.name)
                    .ok_or_else(|| AbiError::MissingParam(param.name.clone()))?
                    .clone();

                value.find_type_mismatch(&param.typ, param.name.clone())?;

                Self::encode_value(value, &param.typ)
            })
            .collect::<Result<_, _>>()?;

        // When encoding public inputs to be passed to the verifier, the user can must provide a return value
        // to be inserted into the witness map. This is not needed when generating a witness when proving the circuit.
        match (&self.return_type, return_value) {
            (Some(AbiReturnType { abi_type: return_type, .. }), Some(return_value)) => {
                if !return_value.matches_abi(return_type) {
                    return Err(AbiError::ReturnTypeMismatch {
                        return_type: return_type.clone(),
                        value: return_value,
                    });
                }
                let encoded_return_fields = Self::encode_value(return_value, return_type)?;
                encoded_inputs.push(encoded_return_fields);
            }
            (None, Some(return_value)) => {
                return Err(AbiError::UnexpectedReturnValue(return_value))
            }
            // We allow not passing a return value despite the circuit defining one
            // in order to generate the initial partial witness.
            (_, None) => {}
        }

        // Write input field elements into witness map.
        let witness_map: BTreeMap<Witness, FieldElement> = encoded_inputs
            .into_iter()
            .flatten()
            .enumerate()
            .map(|(index, field_element)| (Witness(index as u32), field_element))
            .collect::<BTreeMap<Witness, FieldElement>>();

        Ok(witness_map.into())
    }

    fn encode_value(value: InputValue, abi_type: &AbiType) -> Result<Vec<FieldElement>, AbiError> {
        let mut encoded_value = Vec::new();
        match (value, abi_type) {
            (InputValue::Field(elem), _) => encoded_value.push(elem),

            (InputValue::Vec(vec_elements), AbiType::Array { typ, .. }) => {
                for elem in vec_elements {
                    encoded_value.extend(Self::encode_value(elem, typ)?);
                }
            }

            (InputValue::String(string), _) => {
                let str_as_fields =
                    string.bytes().map(|byte| FieldElement::from_be_bytes_reduce(&[byte]));
                encoded_value.extend(str_as_fields);
            }

            (InputValue::Struct(object), AbiType::Struct { fields, .. }) => {
                for (field, typ) in fields {
                    encoded_value.extend(Self::encode_value(object[field].clone(), typ)?);
                }
            }
            (InputValue::Vec(vec_elements), AbiType::Tuple { fields }) => {
                for (value, typ) in vec_elements.into_iter().zip(fields) {
                    encoded_value.extend(Self::encode_value(value, typ)?);
                }
            }
            _ => unreachable!("value should have already been checked to match abi type"),
        }
        Ok(encoded_value)
    }

    /// Decode a `WitnessMap` into the types specified in the ABI.
    pub fn decode(
        &self,
        witness_map: &WitnessMap<FieldElement>,
    ) -> Result<(InputMap, Option<InputValue>), AbiError> {
        let mut pointer: u32 = 0;
        let public_inputs_map =
            try_btree_map(self.parameters.clone(), |AbiParameter { name, typ, .. }| {
                let num_fields = typ.field_count();
                let param_witness_values = try_vecmap(0..num_fields, |index| {
                    let witness_index = Witness(pointer + index);
                    witness_map
                        .get(&witness_index)
                        .ok_or_else(|| AbiError::MissingParamWitnessValue {
                            name: name.clone(),
                            witness_index,
                        })
                        .copied()
                })?;
                pointer += num_fields;

                decode_value(&mut param_witness_values.into_iter(), &typ)
                    .map(|input_value| (name.clone(), input_value))
            })?;

        // We also attempt to decode the circuit's return value from `witness_map`.
        let return_value = if let Some(return_type) = &self.return_type {
            if let Ok(return_witness_values) =
                try_vecmap(0..return_type.abi_type.field_count(), |index| {
                    let witness_index = Witness(pointer + index);
                    witness_map
                        .get(&witness_index)
                        .ok_or_else(|| AbiError::MissingParamWitnessValue {
                            name: MAIN_RETURN_NAME.to_string(),
                            witness_index,
                        })
                        .copied()
                })
            {
                Some(decode_value(&mut return_witness_values.into_iter(), &return_type.abi_type)?)
            } else {
                // Unlike for the circuit inputs, we tolerate not being able to find the witness values for the return value.
                // This is because the user may be decoding a partial witness map for which is hasn't been calculated yet.
                // If a return value is expected, this should be checked for by the user.
                None
            }
        } else {
            None
        };

        Ok((public_inputs_map, return_value))
    }
}

pub fn decode_value(
    field_iterator: &mut impl Iterator<Item = FieldElement>,
    value_type: &AbiType,
) -> Result<InputValue, AbiError> {
    // This function assumes that `field_iterator` contains enough `FieldElement`s in order to decode a `value_type`
    // `Abi.decode` enforces that the encoded inputs matches the expected length defined by the ABI so this is safe.
    let value = match value_type {
        AbiType::Field | AbiType::Integer { .. } | AbiType::Boolean => {
            let field_element = field_iterator.next().unwrap();

            InputValue::Field(field_element)
        }
        AbiType::Array { length, typ } => {
            let length = *length as usize;
            let mut array_elements = Vec::with_capacity(length);
            for _ in 0..length {
                array_elements.push(decode_value(field_iterator, typ)?);
            }

            InputValue::Vec(array_elements)
        }
        AbiType::String { length } => {
            let field_elements: Vec<FieldElement> = field_iterator.take(*length as usize).collect();

            InputValue::String(decode_string_value(&field_elements))
        }
        AbiType::Struct { fields, .. } => {
            let mut struct_map = BTreeMap::new();

            for (field_key, param_type) in fields {
                let field_value = decode_value(field_iterator, param_type)?;

                struct_map.insert(field_key.to_owned(), field_value);
            }

            InputValue::Struct(struct_map)
        }
        AbiType::Tuple { fields } => {
            let mut tuple_elements = Vec::with_capacity(fields.len());
            for field_typ in fields {
                tuple_elements.push(decode_value(field_iterator, field_typ)?);
            }

            InputValue::Vec(tuple_elements)
        }
    };

    Ok(value)
}

fn decode_string_value(field_elements: &[FieldElement]) -> String {
    let string_as_slice = vecmap(field_elements, |e| {
        let mut field_as_bytes = e.to_be_bytes();
        let char_byte = field_as_bytes.pop().unwrap(); // A character in a string is represented by a u8, thus we just want the last byte of the element
        assert!(field_as_bytes.into_iter().all(|b| b == 0)); // Assert that the rest of the field element's bytes are empty
        char_byte
    });

    let final_string = str::from_utf8(&string_as_slice).unwrap();
    final_string.to_owned()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum AbiValue {
    Field {
        value: FieldElement,
    },
    Integer {
        sign: bool,
        value: String,
    },
    Boolean {
        value: bool,
    },
    String {
        value: String,
    },
    Array {
        value: Vec<AbiValue>,
    },
    Struct {
        #[serde(
            serialize_with = "serialization::serialize_struct_field_values",
            deserialize_with = "serialization::deserialize_struct_field_values"
        )]
        fields: Vec<(String, AbiValue)>,
    },
    Tuple {
        fields: Vec<AbiValue>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(tag = "error_kind", rename_all = "lowercase")]
pub enum AbiErrorType {
    FmtString { length: u32, item_types: Vec<AbiType> },
    Custom(AbiType),
    String { string: String },
}

pub fn display_abi_error<F: AcirField>(
    fields: &[F],
    error_type: AbiErrorType,
) -> PrintableValueDisplay<F> {
    match error_type {
        AbiErrorType::FmtString { length, item_types } => {
            let mut fields_iter = fields.iter().copied();
            let PrintableValue::String(string) =
                printable_type_decode_value(&mut fields_iter, &PrintableType::String { length })
            else {
                unreachable!("Got non-string from string decoding");
            };
            let _length_of_items = fields_iter.next();
            let items = item_types.into_iter().map(|abi_type| {
                let printable_typ = (&abi_type).into();
                let decoded = printable_type_decode_value(&mut fields_iter, &printable_typ);
                (decoded, printable_typ)
            });
            PrintableValueDisplay::FmtString(string, items.collect())
        }
        AbiErrorType::Custom(abi_typ) => {
            let printable_type = (&abi_typ).into();
            let decoded = printable_type_decode_value(&mut fields.iter().copied(), &printable_type);
            PrintableValueDisplay::Plain(decoded, printable_type)
        }
        AbiErrorType::String { string } => {
            let length = string.len() as u32;
            PrintableValueDisplay::Plain(
                PrintableValue::String(string),
                PrintableType::String { length },
            )
        }
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;

    use crate::arbitrary::arb_abi_and_input_map;

    proptest! {
        #[test]
        fn encoding_and_decoding_returns_original_witness_map((abi, input_map) in arb_abi_and_input_map()) {
            let witness_map = abi.encode(&input_map, None).unwrap();
            let (decoded_inputs, return_value) = abi.decode(&witness_map).unwrap();

            prop_assert_eq!(decoded_inputs, input_map);
            prop_assert_eq!(return_value, None);
        }
    }
}
