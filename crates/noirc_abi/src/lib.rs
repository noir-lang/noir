use std::{collections::BTreeMap, convert::TryInto};

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
    Struct {
        #[serde(
            serialize_with = "serialization::serialize_struct_fields",
            deserialize_with = "serialization::deserialize_struct_fields"
        )]
        fields: BTreeMap<String, AbiType>,
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
            AbiType::Field | AbiType::Integer { .. } => 1,
            AbiType::Array { length, typ: _ } => *length as usize,
            AbiType::Struct { fields, .. } => fields.len(),
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
    ) -> Result<Vec<FieldElement>, AbiError> {
        // The `Return` parameter is always present in the ABI because the ABI describes
        // the input and output parameters.
        //
        // When the Prover is creating a proof, they will not supply the `Return` value
        // in their TOML file. This means that when we encode the ABI for the Prover,
        // we should skip the `Return` parameter in ABI as this is not supplied by the prover
        // and therefore cannot be encoded.

        let parameters = match (skip_output, self.output_parameter()) {
            (true, _) | (false, None) => self.input_parameters(),
            (false, Some(output_param)) => {
                let mut params = self.input_parameters();
                params.push(output_param);
                params
            }
        };

        let param_names: Vec<&String> =
            parameters.iter().clone().map(|param| &param.name).collect();
        let mut encoded_inputs = Vec::new();

        for param in parameters {
            let value = inputs
                .get(&param.name)
                .ok_or_else(|| AbiError::MissingParam(param.name.to_owned()))?
                .clone();

            if !value.matches_abi(&param.typ) {
                return Err(AbiError::TypeMismatch { param: param.to_owned(), value });
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
        encoded_outputs: &Vec<FieldElement>,
    ) -> Result<BTreeMap<String, InputValue>, AbiError> {
        let input_length: u32 = encoded_inputs.len().try_into().unwrap();
        if input_length != self.field_count() {
            return Err(AbiError::UnexpectedInputLength {
                actual: input_length,
                expected: self.field_count(),
            });
        }

        let mut decoded_inputs = BTreeMap::new();

        // Decode the input values
        //
        {
            let input_parameters = self.input_parameters();

            let mut index = 0;
            for param in &input_parameters {
                let (next_index, decoded_value) =
                    Self::decode_value(index, encoded_inputs, &param.typ)?;

                decoded_inputs.insert(param.name.to_owned(), decoded_value);

                index = next_index;
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

            let (_, decoded_value) = Self::decode_value(0, encoded_outputs, &output_parameter.typ)?;
            decoded_inputs.insert(output_parameter.name.to_owned(), decoded_value);
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
            AbiType::Array { length, .. } => {
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
