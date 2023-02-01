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
    // Parameters in the ABI excluding the return parameter
    pub fn input_parameters(&self) -> Vec<&AbiParameter> {
        self.parameters.iter().filter(|param| param.name != MAIN_RETURN_NAME).collect()
    }

    // Return parameter in the ABI
    pub fn output_parameter(&self) -> Option<&AbiParameter> {
        self.parameters.iter().find(|param| param.name == MAIN_RETURN_NAME)
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
        skip_output: bool,
    ) -> Result<(Vec<FieldElement>, Option<Vec<FieldElement>>), AbiError> {
        // The `Return` parameter is always present in the ABI because the ABI describes
        // the input and output parameters, where `Return` is the output parameter.
        //
        // When the Prover is creating a proof, they will not supply the `Return` value
        // in their TOML file. This means that when we encode the ABI for the Prover,
        // we should skip the `Return` parameter in ABI as this is not supplied by the prover
        // and therefore cannot be encoded.

        // Input parameters
        //
        let mut encoded_inputs = Vec::new();
        {
            for param in self.input_parameters() {
                let value = inputs
                    .get(&param.name)
                    .ok_or_else(|| AbiError::MissingParam(param.name.to_owned()))?
                    .clone();

                if !value.matches_abi(&param.typ) {
                    return Err(AbiError::TypeMismatch { param: param.to_owned(), value });
                }

                encoded_inputs.extend(Self::encode_value(value, &param.name)?);
            }
        }

        // Output parameters
        //
        let mut encoded_output = None;
        {
            if skip_output {
                return Ok((encoded_inputs, encoded_output));
            }

            let output_param = match (skip_output, self.output_parameter()) {
                (true, _) | (false, None) => return Ok((encoded_inputs, encoded_output)),
                (false, Some(output_param)) => output_param,
            };

            let value = inputs
                .get(&output_param.name)
                .ok_or_else(|| AbiError::MissingParam(output_param.name.to_owned()))?
                .clone();

            if !value.matches_abi(&output_param.typ) {
                return Err(AbiError::TypeMismatch { param: output_param.to_owned(), value });
            }

            encoded_output = Some(Self::encode_value(value, &output_param.name)?);
        }

        let all_params: Vec<_> = self
            .input_parameters()
            .into_iter()
            .chain(self.output_parameter().into_iter())
            .collect();

        let param_names: Vec<&String> =
            all_params.iter().clone().map(|param| &param.name).collect();

        // Check that no extra witness values have been provided.
        // Any missing values should be caught by the above for-loop so this only catches extra values.
        if all_params.len() != inputs.len() {
            let unexpected_params: Vec<String> =
                inputs.keys().filter(|param| !param_names.contains(param)).cloned().collect();
            return Err(AbiError::UnexpectedParams(unexpected_params));
        }

        Ok((encoded_inputs, encoded_output))
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
        encoded_inputs: &[FieldElement],
        encoded_outputs: &[FieldElement],
    ) -> Result<BTreeMap<String, InputValue>, AbiError> {
        let input_length: u32 = encoded_inputs
            .len()
            .try_into()
            .expect("number of inputs is expected to be less than u32::MAX");
        let output_length: u32 = encoded_outputs
            .len()
            .try_into()
            .expect("number of inputs is expected to be less than u32::MAX");
        let total_length = input_length + output_length;
        if total_length != self.field_count() {
            return Err(AbiError::UnexpectedInputLength {
                actual: total_length,
                expected: self.field_count(),
            });
        }

        let mut decoded_inputs = BTreeMap::new();
        {
            let input_parameters = self.input_parameters();
            let mut field_iterator = encoded_inputs.iter().cloned();

            for param in input_parameters {
                let decoded_value = Self::decode_value(&mut field_iterator, &param.typ)?;

                decoded_inputs.insert(param.name.to_owned(), decoded_value);
            }
        }

        // Decode the output value
        //
        {
            if encoded_outputs.is_empty() {
                return Ok(decoded_inputs);
            }

            let output_parameter = match self.output_parameter() {
                Some(output) => output,
                None => unreachable!("infallible: the encoded outputs is none empty, but the ABI specifies no return values"),
            };

            let mut field_iterator = encoded_outputs.iter().cloned();

            let decoded_value = Self::decode_value(&mut field_iterator, &output_parameter.typ)?;
            decoded_inputs.insert(output_parameter.name.to_owned(), decoded_value);
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
                let string_as_slice: Vec<u8> = field_iterator
                    .take(*length as usize)
                    .map(|e| {
                        let mut field_as_bytes = e.to_be_bytes();
                        let char_byte = field_as_bytes.pop().unwrap(); // A character in a string is represented by a u8, thus we just want the last byte of the element
                        assert!(field_as_bytes.into_iter().all(|b| b == 0)); // Assert that the rest of the field element's bytes are empty
                        char_byte
                    })
                    .collect();

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
