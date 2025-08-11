//! This file contains mechanisms for deterministically mutating a given ValueType
use crate::mutations::configuration::{ValueTypeMutationConfig, ValueTypeMutationOptions};
use noir_ssa_fuzzer::typed_value::ValueType;
use rand::rngs::StdRng;

pub(crate) fn mutate_value_type(
    value_type: &mut ValueType,
    rng: &mut StdRng,
    config: ValueTypeMutationConfig,
) {
    match config.select(rng) {
        ValueTypeMutationOptions::Field => {
            *value_type = ValueType::Field;
        }
        ValueTypeMutationOptions::Boolean => {
            *value_type = ValueType::Boolean;
        }
        ValueTypeMutationOptions::U8 => {
            *value_type = ValueType::U8;
        }
        ValueTypeMutationOptions::U16 => {
            *value_type = ValueType::U16;
        }
        ValueTypeMutationOptions::U32 => {
            *value_type = ValueType::U32;
        }
        ValueTypeMutationOptions::U64 => {
            *value_type = ValueType::U64;
        }
        ValueTypeMutationOptions::U128 => {
            *value_type = ValueType::U128;
        }
        ValueTypeMutationOptions::I8 => {
            *value_type = ValueType::I8;
        }
        ValueTypeMutationOptions::I16 => {
            *value_type = ValueType::I16;
        }
        ValueTypeMutationOptions::I32 => {
            *value_type = ValueType::I32;
        }
        ValueTypeMutationOptions::I64 => {
            *value_type = ValueType::I64;
        }
    }
}
