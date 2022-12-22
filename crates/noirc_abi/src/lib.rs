use std::{collections::BTreeMap, convert::TryInto};

use acvm::FieldElement;
use errors::AbiError;
use input_parser::InputValue;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// This is the ABI used to bridge the different TOML formats for the initial
// witness, the partial witness generator and the interpreter.
//
// This ABI has nothing to do with ACVM or ACIR. Although they implicitly have a relationship

pub mod errors;
pub mod input_parser;

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
        #[serde(serialize_with = "serialize_struct", deserialize_with = "deserialize_struct")]
        fields: BTreeMap<String, AbiType>,
    },
}

#[derive(Serialize, Deserialize)]
struct StructField {
    name: String,
    #[serde(rename = "type")]
    typ: AbiType,
}

fn serialize_struct<S>(fields: &BTreeMap<String, AbiType>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let fields_vector: Vec<StructField> = fields
        .iter()
        .map(|(name, typ)| StructField { name: name.to_owned(), typ: typ.to_owned() })
        .collect();
    fields_vector.serialize(s)
}

fn deserialize_struct<'de, D>(deserializer: D) -> Result<BTreeMap<String, AbiType>, D::Error>
where
    D: Deserializer<'de>,
{
    let fields_vector = Vec::<StructField>::deserialize(deserializer)?;
    let fields = fields_vector.into_iter().map(|StructField { name, typ }| (name, typ)).collect();
    Ok(fields)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{AbiParameter, AbiType, AbiVisibility, Sign};

    #[test]
    fn abi_parameter_serialization() {
        let serialized_field = "{
            \"name\": \"thing1\",
            \"visibility\": \"public\",
            \"type\": {
                \"kind\": \"field\"
            }
        }";

        let expected_field = AbiParameter {
            name: "thing1".to_string(),
            typ: AbiType::Field,
            visibility: AbiVisibility::Public,
        };
        let deserialized_field: AbiParameter = serde_json::from_str(serialized_field).unwrap();
        assert_eq!(deserialized_field, expected_field);

        let serialized_array = "{
            \"name\": \"thing2\",
            \"visibility\": \"private\",
            \"type\": {
                \"kind\": \"array\",
                \"length\": 2,
                \"type\": {
                    \"kind\": \"integer\",
                    \"width\": 3,
                    \"sign\": \"unsigned\"
                }
            }
        }";

        let expected_array = AbiParameter {
            name: "thing2".to_string(),
            typ: AbiType::Array {
                length: 2,
                typ: Box::new(AbiType::Integer { sign: Sign::Unsigned, width: 3 }),
            },
            visibility: AbiVisibility::Private,
        };
        let deserialized_array: AbiParameter = serde_json::from_str(serialized_array).unwrap();
        assert_eq!(deserialized_array, expected_array);

        let serialized_struct = "{   
            \"name\":\"thing3\",
            \"type\": {
                \"kind\":\"struct\",
                \"fields\": [
                    {
                        \"name\": \"field1\",
                        \"type\": {
                            \"kind\": \"integer\",
                            \"sign\": \"unsigned\",
                            \"width\": 3
                        }
                    },
                    {
                        \"name\":\"field2\",
                        \"type\": {
                            \"kind\":\"array\",
                            \"length\": 2,
                            \"type\": {
                                \"kind\":\"field\"
                            }
                        }
                    }
                ]
            },
            \"visibility\":\"private\"
        }";

        let expected_struct = AbiParameter {
            name: "thing3".to_string(),
            typ: AbiType::Struct {
                fields: BTreeMap::from([
                    ("field1".to_string(), AbiType::Integer { sign: Sign::Unsigned, width: 3 }),
                    (
                        "field2".to_string(),
                        AbiType::Array { length: 2, typ: Box::new(AbiType::Field) },
                    ),
                ]),
            },
            visibility: AbiVisibility::Private,
        };
        let deserialized_struct: AbiParameter = serde_json::from_str(serialized_struct).unwrap();
        assert_eq!(deserialized_struct, expected_struct);
    }
}
