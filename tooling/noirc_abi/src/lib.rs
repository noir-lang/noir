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
use noirc_frontend::ast::{Signedness, Visibility};
use noirc_frontend::{hir::Context, Type, TypeBinding, TypeVariableKind};
use noirc_printable_type::{
    decode_value as printable_type_decode_value, PrintableType, PrintableValue,
    PrintableValueDisplay,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, ops::Range};
use std::{collections::BTreeMap, str};
// This is the ABI used to bridge the different TOML formats for the initial
// witness, the partial witness generator and the interpreter.
//
// This ABI has nothing to do with ACVM or ACIR. Although they implicitly have a relationship

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
    DataBus,
}

impl From<Visibility> for AbiVisibility {
    fn from(value: Visibility) -> Self {
        match value {
            Visibility::Public => AbiVisibility::Public,
            Visibility::Private => AbiVisibility::Private,
            Visibility::DataBus => AbiVisibility::DataBus,
        }
    }
}

impl From<&Visibility> for AbiVisibility {
    fn from(value: &Visibility) -> Self {
        match value {
            Visibility::Public => AbiVisibility::Public,
            Visibility::Private => AbiVisibility::Private,
            Visibility::DataBus => AbiVisibility::DataBus,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Represents whether the return value should compromise of unique witness indices such that no
/// index occurs within the program's abi more than once.
///
/// This is useful for application stacks that require an uniform abi across across multiple
/// circuits. When index duplication is allowed, the compiler may identify that a public input
/// reaches the output unaltered and is thus referenced directly, causing the input and output
/// witness indices to overlap. Similarly, repetitions of copied values in the output may be
/// optimized away.
pub enum AbiDistinctness {
    Distinct,
    DuplicationAllowed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sign {
    Unsigned,
    Signed,
}

impl AbiType {
    pub fn from_type(context: &Context, typ: &Type) -> Self {
        // Note; use strict_eq instead of partial_eq when comparing field types
        // in this method, you most likely want to distinguish between public and private
        match typ {
            Type::FieldElement => Self::Field,
            Type::Array(size, typ) => {
                let length = size
                    .evaluate_to_u64()
                    .expect("Cannot have variable sized arrays as a parameter to main");
                let typ = typ.as_ref();
                Self::Array { length, typ: Box::new(Self::from_type(context, typ)) }
            }
            Type::Integer(sign, bit_width) => {
                let sign = match sign {
                    Signedness::Unsigned => Sign::Unsigned,
                    Signedness::Signed => Sign::Signed,
                };

                Self::Integer { sign, width: (*bit_width).into() }
            }
            Type::TypeVariable(binding, TypeVariableKind::IntegerOrField)
            | Type::TypeVariable(binding, TypeVariableKind::Integer) => match &*binding.borrow() {
                TypeBinding::Bound(typ) => Self::from_type(context, typ),
                TypeBinding::Unbound(_) => {
                    Self::from_type(context, &Type::default_int_or_field_type())
                }
            },
            Type::Bool => Self::Boolean,
            Type::String(size) => {
                let size = size
                    .evaluate_to_u64()
                    .expect("Cannot have variable sized strings as a parameter to main");
                Self::String { length: size }
            }

            Type::Struct(def, args) => {
                let struct_type = def.borrow();
                let fields = struct_type.get_fields(args);
                let fields = vecmap(fields, |(name, typ)| (name, Self::from_type(context, &typ)));
                // For the ABI, we always want to resolve the struct paths from the root crate
                let path =
                    context.fully_qualified_struct_path(context.root_crate_id(), struct_type.id);
                Self::Struct { fields, path }
            }
            Type::Alias(def, args) => Self::from_type(context, &def.borrow().get_type(args)),
            Type::Tuple(fields) => {
                let fields = vecmap(fields, |typ| Self::from_type(context, typ));
                Self::Tuple { fields }
            }
            Type::Error
            | Type::Unit
            | Type::Constant(_)
            | Type::TraitAsType(..)
            | Type::TypeVariable(_, _)
            | Type::NamedGeneric(..)
            | Type::Forall(..)
            | Type::Code
            | Type::Slice(_)
            | Type::Function(_, _, _) => unreachable!("{typ} cannot be used in the abi"),
            Type::FmtString(_, _) => unreachable!("format strings cannot be used in the abi"),
            Type::MutableReference(_) => unreachable!("&mut cannot be used in the abi"),
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
            AbiType::Tuple { fields } => {
                fields.iter().fold(0, |acc, field_typ| acc + field_typ.field_count())
            }
            AbiType::String { length } => *length as u32,
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
pub struct AbiReturnType {
    pub abi_type: AbiType,
    pub visibility: AbiVisibility,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Abi {
    /// An ordered list of the arguments to the program's `main` function, specifying their types and visibility.
    pub parameters: Vec<AbiParameter>,
    /// A map from the ABI's parameters to the indices they are written to in the [`WitnessMap`].
    /// This defines how to convert between the [`InputMap`] and [`WitnessMap`].
    pub param_witnesses: BTreeMap<String, Vec<Range<Witness>>>,
    pub return_type: Option<AbiReturnType>,
    pub return_witnesses: Vec<Witness>,
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
            error_types: self.error_types,
        }
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
        let encoded_input_map: BTreeMap<String, Vec<FieldElement>> = self
            .to_btree_map()
            .into_iter()
            .map(|(param_name, expected_type)| {
                let value = input_map
                    .get(&param_name)
                    .ok_or_else(|| AbiError::MissingParam(param_name.clone()))?
                    .clone();

                value.find_type_mismatch(&expected_type, param_name.clone())?;

                Self::encode_value(value, &expected_type).map(|v| (param_name, v))
            })
            .collect::<Result<_, _>>()?;

        // Write input field elements into witness indices specified in `self.param_witnesses`.
        let mut witness_map: BTreeMap<Witness, FieldElement> = encoded_input_map
            .iter()
            .flat_map(|(param_name, encoded_param_fields)| {
                let param_witness_indices = range_to_vec(&self.param_witnesses[param_name]);
                param_witness_indices
                    .iter()
                    .zip(encoded_param_fields.iter())
                    .map(|(&witness, &field_element)| (witness, field_element))
                    .collect::<Vec<_>>()
            })
            .collect::<BTreeMap<Witness, FieldElement>>();

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
        let public_inputs_map =
            try_btree_map(self.parameters.clone(), |AbiParameter { name, typ, .. }| {
                let param_witness_values =
                    try_vecmap(range_to_vec(&self.param_witnesses[&name]), |witness_index| {
                        witness_map
                            .get(&witness_index)
                            .ok_or_else(|| AbiError::MissingParamWitnessValue {
                                name: name.clone(),
                                witness_index,
                            })
                            .copied()
                    })?;

                decode_value(&mut param_witness_values.into_iter(), &typ)
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
                // We do not return value for the data bus.
                if return_type.visibility == AbiVisibility::DataBus {
                    None
                } else {
                    Some(decode_value(
                        &mut return_witness_values.into_iter(),
                        &return_type.abi_type,
                    )?)
                }
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

fn range_to_vec(ranges: &[Range<Witness>]) -> Vec<Witness> {
    let mut result = Vec::new();
    for range in ranges {
        for witness in range.start.witness_index()..range.end.witness_index() {
            result.push(witness.into());
        }
    }
    result
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error_kind", rename_all = "lowercase")]
pub enum AbiErrorType {
    FmtString { length: u64, item_types: Vec<AbiType> },
    Custom(AbiType),
}
impl AbiErrorType {
    pub fn from_type(context: &Context, typ: &Type) -> Self {
        match typ {
            Type::FmtString(len, item_types) => {
                let length = len.evaluate_to_u64().expect("Cannot evaluate fmt length");
                let Type::Tuple(item_types) = item_types.as_ref() else {
                    unreachable!("FmtString items must be a tuple")
                };
                let item_types =
                    item_types.iter().map(|typ| AbiType::from_type(context, typ)).collect();
                Self::FmtString { length, item_types }
            }
            _ => Self::Custom(AbiType::from_type(context, typ)),
        }
    }
}

pub fn display_abi_error(
    fields: &[FieldElement],
    error_type: AbiErrorType,
) -> PrintableValueDisplay {
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
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use acvm::{acir::native_types::Witness, AcirField, FieldElement};

    use crate::{
        input_parser::InputValue, Abi, AbiParameter, AbiReturnType, AbiType, AbiVisibility,
        InputMap,
    };

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
                ("thing1".to_string(), vec![(Witness(1)..Witness(3))]),
                ("thing2".to_string(), vec![(Witness(3)..Witness(4))]),
            ]),
            return_type: Some(AbiReturnType {
                abi_type: AbiType::Field,
                visibility: AbiVisibility::Public,
            }),
            return_witnesses: vec![Witness(3)],
            error_types: BTreeMap::default(),
        };

        // Note we omit return value from inputs
        let inputs: InputMap = BTreeMap::from([
            (
                "thing1".to_string(),
                InputValue::Vec(vec![
                    InputValue::Field(FieldElement::one()),
                    InputValue::Field(FieldElement::one()),
                ]),
            ),
            ("thing2".to_string(), InputValue::Field(FieldElement::zero())),
        ]);

        let witness_map = abi.encode(&inputs, None).unwrap();
        let (reconstructed_inputs, return_value) = abi.decode(&witness_map).unwrap();

        for (key, expected_value) in inputs {
            assert_eq!(reconstructed_inputs[&key], expected_value);
        }

        // We also decode the return value (we can do this immediately as we know it shares a witness with an input).
        assert_eq!(return_value.unwrap(), reconstructed_inputs["thing2"]);
    }
}
