//! This file contains mechanisms for deterministically mutating a given ValueType
use crate::mutations::configuration::{
    GenerateValueType, GenerateValueTypeConfig, ValueTypeMutationConfig, ValueTypeMutationOptions,
};
use noir_ssa_fuzzer::typed_value::ValueType;
use rand::rngs::StdRng;

pub(crate) fn generate_random_value_type(
    rng: &mut StdRng,
    config: GenerateValueTypeConfig,
) -> ValueType {
    match config.select(rng) {
        GenerateValueType::Field => ValueType::Field,
        GenerateValueType::Boolean => ValueType::Boolean,
        GenerateValueType::U8 => ValueType::U8,
        GenerateValueType::U16 => ValueType::U16,
        GenerateValueType::U32 => ValueType::U32,
        GenerateValueType::U64 => ValueType::U64,
        GenerateValueType::U128 => ValueType::U128,
        GenerateValueType::I8 => ValueType::I8,
        GenerateValueType::I16 => ValueType::I16,
        GenerateValueType::I32 => ValueType::I32,
        GenerateValueType::I64 => ValueType::I64,
    }
}

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
