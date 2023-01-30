use std::{collections::BTreeMap, convert::TryInto, str};

use acvm::FieldElement;
use errors::AbiError;
use input_parser::InputValue;
use serde::{Deserialize, Serialize};

// This is the ABI used to bridge the different TOML formats for the initial
// witness, the partial witness generator and the interpreter.
//
// This ABI has nothing to do with ACVM or ACIR. Although they implicitly have a relationship

pub mod errors;
pub mod input_parser;
mod serialization;

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
pub enum AbiType {
    Field,
    Array {
        length: u64,
        #[serde(rename = "type")]
        typ: Box<AbiType>,
    },
    Integer {
        sign: Sign,
        width: u32,
    },
    Boolean,
    Struct {
        #[serde(
            serialize_with = "serialization::serialize_struct_fields",
            deserialize_with = "serialization::deserialize_struct_fields"
        )]
        fields: BTreeMap<String, AbiType>,
    },
    String {
        length: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Represents whether the parameter is public or known only to the prover.
pub enum AbiVisibility {
    Public,
    // Constants are not allowed in the ABI for main at the moment.
    // Constant,
    Private,
}

impl std::fmt::Display for AbiVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AbiVisibility::Public => write!(f, "pub"),
            AbiVisibility::Private => write!(f, "priv"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sign {
    Unsigned,
    Signed,
}

impl AbiType {
    pub fn num_elements(&self) -> usize {
        match self {
            AbiType::Field | AbiType::Integer { .. } | AbiType::Boolean => 1,
            AbiType::Array { length, typ: _ } => *length as usize,
            AbiType::Struct { fields, .. } => fields.len(),
            AbiType::String { length } => *length as usize,
        }
    }

    /// Returns the number of field elements required to represent the type once encoded.
    pub fn field_count(&self) -> u32 {
        match self {
            AbiType::Field | AbiType::Integer { .. } | AbiType::Boolean => 1,
            AbiType::Array { length, typ } => typ.field_count() * (*length as u32),
            AbiType::Struct { fields, .. } => {
                fields.iter().fold(0, |acc, (_, field_type)| acc + field_type.field_count())
            }
            AbiType::String { length } => *length as u32,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// An argument or return value of the circuit's `main` function.
pub struct AbiParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub typ: AbiType,
    pub visibility: AbiVisibility,
}

impl AbiParameter {
    pub fn is_public(&self) -> bool {
        self.visibility == AbiVisibility::Public
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Abi {
    pub parameters: Vec<AbiParameter>,
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

    pub fn to_btree_map(&self) -> BTreeMap<String, AbiType> {
        let mut map = BTreeMap::new();
        for param in self.parameters.iter() {
            map.insert(param.name.clone(), param.typ.clone());
        }
        map
    }

    /// ABI with only the public parameters
    #[must_use]
    pub fn public_abi(self) -> Abi {
        let parameters: Vec<_> =
            self.parameters.into_iter().filter(|param| param.is_public()).collect();
        Abi { parameters }
    }

    /// Encode a set of inputs as described in the ABI into a vector of `FieldElement`s.
    pub fn encode(
        self,
        inputs: &BTreeMap<String, InputValue>,
        allow_undefined_return: bool,
    ) -> Result<Vec<FieldElement>, AbiError> {
        let param_names = self.parameter_names();
        let mut encoded_inputs = Vec::new();

        for param in self.parameters.iter() {
            let value = inputs
                .get(&param.name)
                .ok_or_else(|| AbiError::MissingParam(param.name.to_owned()))?
                .clone();

            if !value.matches_abi(&param.typ) {
                return Err(AbiError::TypeMismatch { param: param.to_owned(), value });
            }

            // As the circuit calculates the return value in the process of calculating rest of the witnesses
            // it's not absolutely necessary to provide them as inputs. We then tolerate an undefined value for
            // the return value input and just skip it.
            if allow_undefined_return
                && param.name == MAIN_RETURN_NAME
                && matches!(value, InputValue::Undefined)
            {
                let return_witness_len = param.typ.field_count();

                // We do not support undefined arrays for now - TODO
                if return_witness_len != 1 {
                    return Err(AbiError::Generic(
                        "Values of array returned from main must be specified".to_string(),
                    ));
                } else {
                    // This assumes that the return value is at the end of the ABI, otherwise values will be misaligned.
                    continue;
                }
            }

            encoded_inputs.extend(Self::encode_value(value, &param.name)?);
        }

        // Check that no extra witness values have been provided.
        // Any missing values should be caught by the above for-loop so this only catches extra values.
        if param_names.len() != inputs.len() {
            let unexpected_params: Vec<String> =
                inputs.keys().filter(|param| !param_names.contains(param)).cloned().collect();
            return Err(AbiError::UnexpectedParams(unexpected_params));
        }

        Ok(encoded_inputs)
    }

    fn encode_value(value: InputValue, param_name: &String) -> Result<Vec<FieldElement>, AbiError> {
        let mut encoded_value = Vec::new();
        match value {
            InputValue::Field(elem) => encoded_value.push(elem),
            InputValue::Vec(vec_elem) => encoded_value.extend(vec_elem),
            InputValue::String(string) => {
                let str_as_fields =
                    string.bytes().map(|byte| FieldElement::from_be_bytes_reduce(&[byte]));
                encoded_value.extend(str_as_fields)
            }
            InputValue::Struct(object) => {
                for (field_name, value) in object {
                    let new_name = format!("{param_name}.{field_name}");
                    encoded_value.extend(Self::encode_value(value, &new_name)?)
                }
            }
            InputValue::Undefined => return Err(AbiError::UndefinedInput(param_name.to_string())),
        }
        Ok(encoded_value)
    }

    /// Decode a vector of `FieldElements` into the types specified in the ABI.
    pub fn decode(
        &self,
        encoded_inputs: &Vec<FieldElement>,
    ) -> Result<BTreeMap<String, InputValue>, AbiError> {
        let input_length: u32 = encoded_inputs.len().try_into().unwrap();
        if input_length != self.field_count() {
            return Err(AbiError::UnexpectedInputLength {
                actual: input_length,
                expected: self.field_count(),
            });
        }

        let mut field_iterator = encoded_inputs.iter().cloned();
        let mut decoded_inputs = BTreeMap::new();

        for param in &self.parameters {
            let decoded_value = Self::decode_value(&mut field_iterator, &param.typ)?;

            decoded_inputs.insert(param.name.to_owned(), decoded_value);
        }
        Ok(decoded_inputs)
    }

    fn decode_value(
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
            AbiType::Array { length, .. } => {
                let field_elements: Vec<FieldElement> =
                    field_iterator.take(*length as usize).collect();

                InputValue::Vec(field_elements)
            }
            AbiType::String { length } => {
                let field_elements: Vec<FieldElement> =
                    field_iterator.take(*length as usize).collect();

                let string_as_slice = field_elements
                    .iter()
                    .map(|e| {
                        let mut field_as_bytes = e.to_be_bytes();
                        let char_byte = field_as_bytes.pop().unwrap(); // A character in a string is represented by a u8, thus we just want the last byte of the element
                        assert!(field_as_bytes.into_iter().all(|b| b == 0)); // Assert that the rest of the field element's bytes are empty
                        char_byte
                    })
                    .collect::<Vec<_>>();

                let final_string = str::from_utf8(&string_as_slice).unwrap();

                InputValue::String(final_string.to_owned())
            }
            AbiType::Struct { fields, .. } => {
                let mut struct_map = BTreeMap::new();

                for (field_key, param_type) in fields {
                    let field_value = Self::decode_value(field_iterator, param_type)?;

                    struct_map.insert(field_key.to_owned(), field_value);
                }

                InputValue::Struct(struct_map)
            }
        };

        Ok(value)
    }
}
