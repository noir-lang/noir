//! This file contains mechanisms for deterministically mutating a given ValueType
use crate::mutations::configuration::{
    GenerateNumericType, GenerateNumericTypeConfig, NumericTypeMutationConfig,
    NumericTypeMutationOptions,
};
use noir_ssa_fuzzer::typed_value::NumericType;
use rand::rngs::StdRng;

pub(crate) fn generate_random_numeric_type(
    rng: &mut StdRng,
    config: GenerateNumericTypeConfig,
) -> NumericType {
    match config.select(rng) {
        GenerateNumericType::Field => NumericType::Field,
        GenerateNumericType::Boolean => NumericType::Boolean,
        GenerateNumericType::U8 => NumericType::U8,
        GenerateNumericType::U16 => NumericType::U16,
        GenerateNumericType::U32 => NumericType::U32,
        GenerateNumericType::U64 => NumericType::U64,
        GenerateNumericType::U128 => NumericType::U128,
        GenerateNumericType::I8 => NumericType::I8,
        GenerateNumericType::I16 => NumericType::I16,
        GenerateNumericType::I32 => NumericType::I32,
        GenerateNumericType::I64 => NumericType::I64,
    }
}

pub(crate) fn mutate_numeric_type(
    numeric_type: &mut NumericType,
    rng: &mut StdRng,
    config: NumericTypeMutationConfig,
) {
    match config.select(rng) {
        NumericTypeMutationOptions::Field => {
            *numeric_type = NumericType::Field;
        }
        NumericTypeMutationOptions::Boolean => {
            *numeric_type = NumericType::Boolean;
        }
        NumericTypeMutationOptions::U8 => {
            *numeric_type = NumericType::U8;
        }
        NumericTypeMutationOptions::U16 => {
            *numeric_type = NumericType::U16;
        }
        NumericTypeMutationOptions::U32 => {
            *numeric_type = NumericType::U32;
        }
        NumericTypeMutationOptions::U64 => {
            *numeric_type = NumericType::U64;
        }
        NumericTypeMutationOptions::U128 => {
            *numeric_type = NumericType::U128;
        }
        NumericTypeMutationOptions::I8 => {
            *numeric_type = NumericType::I8;
        }
        NumericTypeMutationOptions::I16 => {
            *numeric_type = NumericType::I16;
        }
        NumericTypeMutationOptions::I32 => {
            *numeric_type = NumericType::I32;
        }
        NumericTypeMutationOptions::I64 => {
            *numeric_type = NumericType::I64;
        }
    }
}
