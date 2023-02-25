#![forbid(unsafe_code)]
#![warn(unreachable_pub)]

use std::{collections::BTreeMap, str};

use acvm::{acir::native_types::Witness, FieldElement};
use errors::AbiError;
use input_parser::InputValue;
use iter_extended::{try_btree_map, try_vecmap, vecmap};
use serde::{Deserialize, Serialize};
// This is the ABI used to bridge the different TOML formats for the initial
// witness, the partial witness generator and the interpreter.
//
// This ABI has nothing to do with ACVM or ACIR. Although they implicitly have a relationship

pub mod errors;
pub mod input_parser;
mod serialization;

/// A map from the fields in an TOML/JSON file which correspond to some ABI to their values
pub type InputMap = BTreeMap<String, InputValue>;

/// A map from the witnesses in a constraint system to the field element values
pub type WitnessMap = BTreeMap<Witness, FieldElement>;

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
    /// An ordered list of the arguments to the program's `main` function, specifying their types and visibility.
    pub parameters: Vec<AbiParameter>,
    /// A map from the ABI's parameters to the indices they are written to in the [`WitnessMap`].
    /// This defines how to convert between the [`InputMap`] and [`WitnessMap`].
    pub param_witnesses: BTreeMap<String, Vec<Witness>>,
    pub return_type: Option<AbiType>,
    pub return_witnesses: Vec<Witness>,
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
        self.return_type.is_some() || self.parameters.iter().any(|param| param.is_public())
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

    /// ABI with only the public parameters
    #[must_use]
    pub fn public_abi(self) -> Abi {
        let parameters: Vec<_> =
            self.parameters.into_iter().filter(|param| param.is_public()).collect();
        let param_witnesses = self
            .param_witnesses
            .into_iter()
            .filter(|(param_name, _)| parameters.iter().any(|param| &param.name == param_name))
            .collect();
        Abi {
            parameters,
            param_witnesses,
            return_type: self.return_type,
            return_witnesses: self.return_witnesses,
        }
    }

    /// Encode a set of inputs as described in the ABI into a `WitnessMap`.
    pub fn encode(
        &self,
        input_map: &InputMap,
        return_value: Option<InputValue>,
    ) -> Result<WitnessMap, AbiError> {
        // Check that no extra witness values have been provided.
        let param_names = self.parameter_names();
        if param_names.len() < input_map.len() {
            let unexpected_params: Vec<String> =
                input_map.keys().filter(|param| !param_names.contains(param)).cloned().collect();
            return Err(AbiError::UnexpectedParams(unexpected_params));
        }

        // First encode each input separately, performing any input validation.
        let encoded_input_map: BTreeMap<String, Vec<FieldElement>> = self
            .to_btree_map()
            .into_iter()
            .map(|(param_name, expected_type)| {
                let value = input_map
                    .get(&param_name)
                    .ok_or_else(|| AbiError::MissingParam(param_name.clone()))?
                    .clone();

                if !value.matches_abi(&expected_type) {
                    let param = self
                        .parameters
                        .iter()
                        .find(|param| param.name == param_name)
                        .unwrap()
                        .clone();
                    return Err(AbiError::TypeMismatch { param, value });
                }

                Self::encode_value(value).map(|v| (param_name, v))
            })
            .collect::<Result<_, _>>()?;

        // Write input field elements into witness indices specified in `self.param_witnesses`.
        let mut witness_map: WitnessMap = encoded_input_map
            .iter()
            .flat_map(|(param_name, encoded_param_fields)| {
                let param_witness_indices = &self.param_witnesses[param_name];
                param_witness_indices
                    .iter()
                    .zip(encoded_param_fields.iter())
                    .map(|(&witness, &field_element)| (witness, field_element))
            })
            .collect();

        // When encoding public inputs to be passed to the verifier, the user can must provide a return value
        // to be inserted into the witness map. This is not needed when generating a witness when proving the circuit.
        match (&self.return_type, return_value) {
            (Some(return_type), Some(return_value)) => {
                if !return_value.matches_abi(return_type) {
                    return Err(AbiError::ReturnTypeMismatch {
                        return_type: return_type.clone(),
                        value: return_value,
                    });
                }
                let encoded_return_fields = Self::encode_value(return_value)?;

                // We need to be more careful when writing the return value's witness values.
                // This is as it may share witness indices with other public inputs so we must check that when
                // this occurs the witness values are consistent with each other.
                self.return_witnesses.iter().zip(encoded_return_fields.iter()).try_for_each(
                    |(&witness, &field_element)| match witness_map.insert(witness, field_element) {
                        Some(existing_value) if existing_value != field_element => {
                            Err(AbiError::InconsistentWitnessAssignment(witness))
                        }
                        _ => Ok(()),
                    },
                )?;
            }
            (None, Some(return_value)) => {
                return Err(AbiError::UnexpectedReturnValue(return_value))
            }
            // We allow not passing a return value despite the circuit defining one
            // in order to generate the initial partial witness.
            (_, None) => {}
        }

        Ok(witness_map)
    }

    fn encode_value(value: InputValue) -> Result<Vec<FieldElement>, AbiError> {
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
                for value in object.into_values() {
                    encoded_value.extend(Self::encode_value(value)?)
                }
            }
        }
        Ok(encoded_value)
    }

    /// Decode a `WitnessMap` into the types specified in the ABI.
    pub fn decode(
        &self,
        witness_map: &WitnessMap,
    ) -> Result<(InputMap, Option<InputValue>), AbiError> {
        let public_inputs_map =
            try_btree_map(self.parameters.clone(), |AbiParameter { name, typ, .. }| {
                let param_witness_values =
                    try_vecmap(self.param_witnesses[&name].clone(), |witness_index| {
                        witness_map
                            .get(&witness_index)
                            .ok_or_else(|| AbiError::MissingParamWitnessValue {
                                name: name.clone(),
                                witness_index,
                            })
                            .copied()
                    })?;

                Self::decode_value(&mut param_witness_values.into_iter(), &typ)
                    .map(|input_value| (name.clone(), input_value))
            })?;

        // We also attempt to decode the circuit's return value from `witness_map`.
        let return_value = if let Some(return_type) = &self.return_type {
            if let Ok(return_witness_values) =
                try_vecmap(self.return_witnesses.clone(), |witness_index| {
                    witness_map
                        .get(&witness_index)
                        .ok_or_else(|| AbiError::MissingParamWitnessValue {
                            name: MAIN_RETURN_NAME.to_string(),
                            witness_index,
                        })
                        .copied()
                })
            {
                Some(Self::decode_value(&mut return_witness_values.into_iter(), return_type)?)
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

                InputValue::String(decode_string_value(&field_elements))
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

pub fn decode_string_value(field_elements: &[FieldElement]) -> String {
    let string_as_slice = vecmap(field_elements, |e| {
        let mut field_as_bytes = e.to_be_bytes();
        let char_byte = field_as_bytes.pop().unwrap(); // A character in a string is represented by a u8, thus we just want the last byte of the element
        assert!(field_as_bytes.into_iter().all(|b| b == 0)); // Assert that the rest of the field element's bytes are empty
        char_byte
    });

    let final_string = str::from_utf8(&string_as_slice).unwrap();
    final_string.to_owned()
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use acvm::{acir::native_types::Witness, FieldElement};

    use crate::{input_parser::InputValue, Abi, AbiParameter, AbiType, AbiVisibility, InputMap};

    #[test]
    fn witness_encoding_roundtrip() {
        let abi = Abi {
            parameters: vec![
                AbiParameter {
                    name: "thing1".to_string(),
                    typ: AbiType::Array { length: 2, typ: Box::new(AbiType::Field) },
                    visibility: AbiVisibility::Public,
                },
                AbiParameter {
                    name: "thing2".to_string(),
                    typ: AbiType::Field,
                    visibility: AbiVisibility::Public,
                },
            ],
            // Note that the return value shares a witness with `thing2`
            param_witnesses: BTreeMap::from([
                ("thing1".to_string(), vec![Witness(1), Witness(2)]),
                ("thing2".to_string(), vec![Witness(3)]),
            ]),
            return_type: Some(AbiType::Field),
            return_witnesses: vec![Witness(3)],
        };

        // Note we omit return value from inputs
        let inputs: InputMap = BTreeMap::from([
            ("thing1".to_string(), InputValue::Vec(vec![FieldElement::one(), FieldElement::one()])),
            ("thing2".to_string(), InputValue::Field(FieldElement::zero())),
        ]);

        let witness_map = abi.encode(&inputs, None).unwrap();
        let (reconstructed_inputs, return_value) = abi.decode(&witness_map).unwrap();

        for (key, expected_value) in inputs {
            assert_eq!(reconstructed_inputs[&key], expected_value);
        }

        // We also decode the return value (we can do this immediately as we know it shares a witness with an input).
        assert_eq!(return_value.unwrap(), reconstructed_inputs["thing2"])
    }
}
