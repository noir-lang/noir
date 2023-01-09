use std::{collections::BTreeMap, convert::TryInto};

use acvm::FieldElement;
use errors::AbiError;
use input_parser::InputValue;
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};

// This is the ABI used to bridge the different TOML formats for the initial
// witness, the partial witness generator and the interpreter.
//
// This ABI has nothing to do with ACVM or ACIR. Although they implicitly have a relationship

pub mod errors;
pub mod input_parser;

pub const MAIN_RETURN_NAME: &str = "return";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Types that are allowed in the (main function in binary)
///
/// we use this separation so that we can have types like St∫rings
/// without needing to introduce this in the Noir types
///
/// NOTE: If Strings are introduced as a native type, the translation will
/// be straightforward. Whether exotic types like String will be natively supported
/// depends on the types of programs that users want to do. I don't envision string manipulation
/// in programs, however it is possible to support, with many complications like encoding character set
/// support.
pub enum AbiType {
    Field,
    Array { length: u128, typ: Box<AbiType> },
    Integer { sign: Sign, width: u32 },
    Struct { fields: BTreeMap<String, AbiType> },
    String { length: u128 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
pub enum Sign {
    Unsigned,
    Signed,
}

impl AbiType {
    pub fn num_elements(&self) -> usize {
        match self {
            AbiType::Field | AbiType::Integer { .. } => 1,
            AbiType::Array { length, typ: _ } => *length as usize,
            AbiType::Struct { fields, .. } => fields.len(),
            AbiType::String { length } => *length as usize,
        }
    }

    /// Returns the number of field elements required to represent the type once encoded.
    pub fn field_count(&self) -> u32 {
        match self {
            AbiType::Field | AbiType::Integer { .. } => 1,
            AbiType::Array { length, typ } => typ.field_count() * (*length as u32),
            AbiType::Struct { fields, .. } => {
                fields.iter().fold(0, |acc, (_, field_type)| acc + field_type.field_count())
            }
            AbiType::String { length } => *length as u32,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
/// An argument or return value of the circuit's `main` function.
pub struct AbiParameter {
    pub name: String,
    pub typ: AbiType,
    pub visibility: AbiVisibility,
}

impl AbiParameter {
    pub fn is_public(&self) -> bool {
        self.visibility == AbiVisibility::Public
    }
}

#[derive(Clone, Debug, Deserialize)]
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
                let str_as_fields = string
                    .clone()
                    .into_bytes()
                    .into_iter()
                    .map(|byte| FieldElement::from_be_bytes_reduce(&[byte]))
                    .collect::<Vec<_>>();
                encoded_value.extend(str_as_fields)
            }
            InputValue::Struct(object) => {
                for (field_name, value) in object {
                    let new_name = format!("{}.{}", param_name, field_name);
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

        let mut index = 0;
        let mut decoded_inputs = BTreeMap::new();

        for param in &self.parameters {
            let (next_index, decoded_value) =
                Self::decode_value(index, encoded_inputs, &param.typ)?;

            decoded_inputs.insert(param.name.to_owned(), decoded_value);

            index = next_index;
        }
        Ok(decoded_inputs)
    }

    fn decode_value(
        initial_index: usize,
        encoded_inputs: &Vec<FieldElement>,
        value_type: &AbiType,
    ) -> Result<(usize, InputValue), AbiError> {
        let mut index = initial_index;

        let value = match value_type {
            AbiType::Field | AbiType::Integer { .. } => {
                let field_element = encoded_inputs[index];
                index += 1;

                InputValue::Field(field_element)
            }
            AbiType::Array { length, .. } | AbiType::String { length } => {
                // TODO need to separate String decoding from arrays
                let field_elements = &encoded_inputs[index..index + (*length as usize)];

                index += *length as usize;
                InputValue::Vec(field_elements.to_vec())
            }
            AbiType::Struct { fields, .. } => {
                let mut struct_map = BTreeMap::new();

                for (field_key, param_type) in fields {
                    let (next_index, field_value) =
                        Self::decode_value(index, encoded_inputs, param_type)?;

                    struct_map.insert(field_key.to_owned(), field_value);
                    index = next_index;
                }

                InputValue::Struct(struct_map)
            }
        };

        Ok((index, value))
    }
}

impl Serialize for Abi {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let vec: Vec<u8> = Vec::new();
        let mut map = serializer.serialize_map(Some(self.parameters.len()))?;
        for param in &self.parameters {
            match param.typ {
                AbiType::Field => map.serialize_entry(&param.name, "")?,
                AbiType::Array { .. } | AbiType::String { .. } => {
                    map.serialize_entry(&param.name, &vec)?
                }
                AbiType::Integer { .. } => map.serialize_entry(&param.name, "")?,
                AbiType::Struct { .. } => map.serialize_entry(&param.name, "")?,
            };
        }
        map.end()
    }
}
