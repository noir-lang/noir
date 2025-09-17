//! This file describes initial witness passed to the program

use super::fuzzer::FuzzerData;
use acvm::FieldElement;
use acvm::acir::native_types::{Witness, WitnessMap};
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noir_ssa_fuzzer::typed_value::{NumericType, Type};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Field modulus has 254 bits, and FieldElement::from supports u128, so we use two unsigned integers to represent a field element
/// field = low + high * 2^128
#[derive(Debug, Clone, Copy, Hash, Arbitrary, Serialize, Deserialize)]
pub(crate) struct FieldRepresentation {
    pub(crate) high: u128,
    pub(crate) low: u128,
}

impl From<&FieldRepresentation> for FieldElement {
    fn from(field: &FieldRepresentation) -> FieldElement {
        let lower = FieldElement::from(field.low);
        let upper = FieldElement::from(field.high);
        lower + upper * (FieldElement::from(u128::MAX) + FieldElement::from(1_u128))
    }
}

#[derive(Debug, Clone, Copy, Hash, Arbitrary, Serialize, Deserialize)]
pub(crate) enum WitnessValueNumeric {
    Field(FieldRepresentation),
    U128(u128),
    U64(u64),
    U32(u32),
    U16(u16),
    U8(u8),
    Boolean(bool),
    I64(u64),
    I32(u32),
    I16(u16),
    I8(u8),
}

impl From<WitnessValueNumeric> for NumericType {
    fn from(value: WitnessValueNumeric) -> Self {
        match value {
            WitnessValueNumeric::Field(_) => NumericType::Field,
            WitnessValueNumeric::U128(_) => NumericType::U128,
            WitnessValueNumeric::U64(_) => NumericType::U64,
            WitnessValueNumeric::U32(_) => NumericType::U32,
            WitnessValueNumeric::U16(_) => NumericType::U16,
            WitnessValueNumeric::U8(_) => NumericType::U8,
            WitnessValueNumeric::Boolean(_) => NumericType::Boolean,
            WitnessValueNumeric::I64(_) => NumericType::I64,
            WitnessValueNumeric::I32(_) => NumericType::I32,
            WitnessValueNumeric::I16(_) => NumericType::I16,
            WitnessValueNumeric::I8(_) => NumericType::I8,
        }
    }
}

impl From<WitnessValueNumeric> for FieldElement {
    fn from(value: WitnessValueNumeric) -> Self {
        match value {
            WitnessValueNumeric::Field(field) => FieldElement::from(&field),
            WitnessValueNumeric::U128(u128) => FieldElement::from(u128),
            WitnessValueNumeric::U64(u64) => FieldElement::from(u64),
            WitnessValueNumeric::U32(u32) => FieldElement::from(u32),
            WitnessValueNumeric::U16(u16) => FieldElement::from(u64::from(u16)),
            WitnessValueNumeric::U8(u8) => FieldElement::from(u64::from(u8)),
            WitnessValueNumeric::Boolean(bool) => FieldElement::from(bool),
            WitnessValueNumeric::I64(i64) => FieldElement::from(i64),
            WitnessValueNumeric::I32(i32) => FieldElement::from(u64::from(i32)),
            WitnessValueNumeric::I16(i16) => FieldElement::from(u64::from(i16)),
            WitnessValueNumeric::I8(i8) => FieldElement::from(u64::from(i8)),
        }
    }
}

impl Default for WitnessValueNumeric {
    fn default() -> Self {
        WitnessValueNumeric::Field(FieldRepresentation { high: 0, low: 0 })
    }
}

#[derive(Debug, Clone, Hash, Arbitrary, Serialize, Deserialize)]
pub(crate) enum WitnessValue {
    Numeric(WitnessValueNumeric),
    Array(Vec<WitnessValue>),
}

impl Default for WitnessValue {
    fn default() -> Self {
        WitnessValue::Numeric(WitnessValueNumeric::default())
    }
}

impl WitnessValue {
    fn to_ssa_type(&self) -> Type {
        match self {
            WitnessValue::Numeric(numeric) => Type::Numeric(NumericType::from(*numeric)),
            WitnessValue::Array(arr) => {
                let ssa_type = arr
                    .iter()
                    .map(|v| v.to_ssa_type())
                    .reduce(|a, b| {
                        assert_eq!(a, b, "All SSA types in the array must be the same");
                        a
                    })
                    .unwrap();
                Type::Array(Arc::new(vec![ssa_type; 1]), arr.len() as u32)
            }
        }
    }
}

fn initialize_witness_map_internal(witness: &[WitnessValue]) -> (Vec<FieldElement>, Vec<Type>) {
    let mut types = vec![];
    let mut witness_vec = vec![];
    for witness_value in witness.iter() {
        match witness_value {
            WitnessValue::Numeric(numeric) => {
                witness_vec.push(FieldElement::from(*numeric));
                types.push(witness_value.to_ssa_type());
            }
            WitnessValue::Array(arr) => {
                let type_ = witness_value.to_ssa_type();
                types.push(type_);
                for val in arr {
                    // types of inner arrays are ignored, because they are already added to the types vector
                    let (values, _types) =
                        initialize_witness_map_internal(std::slice::from_ref(val));
                    witness_vec.extend(values);
                }
            }
        };
    }
    (witness_vec, types)
}

/// Initializes [`WitnessMap`] from [`WitnessValue`]
pub(crate) fn initialize_witness_map(
    initial_witness: &[WitnessValue],
) -> (WitnessMap<FieldElement>, Vec<FieldElement>, Vec<Type>) {
    let (mut witness_vec, mut types) = initialize_witness_map_internal(initial_witness);
    // add true and false boolean values
    witness_vec.push(FieldElement::from(1_u32));
    types.push(Type::Numeric(NumericType::Boolean));
    witness_vec.push(FieldElement::from(0_u32));
    types.push(Type::Numeric(NumericType::Boolean));
    let mut witness_map = WitnessMap::new();
    for (i, value) in witness_vec.iter().enumerate() {
        witness_map.insert(Witness(i as u32), *value);
    }
    (witness_map, witness_vec, types)
}

/// Ensures that boolean is defined in all functions
///
/// If boolean is not defined in the function, it is added to the input types
pub(crate) fn ensure_boolean_defined_in_all_functions(data: &mut FuzzerData) {
    for func in &mut data.functions {
        let boolean_presented_in_input_types =
            func.input_types.iter().any(|t| t == &Type::Numeric(NumericType::Boolean));
        if !boolean_presented_in_input_types {
            func.input_types.push(Type::Numeric(NumericType::Boolean));
        }
    }
}
