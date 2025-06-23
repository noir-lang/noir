//! This file contains mechanisms for deterministically mutating a given [WitnessValue](crate::fuzz_lib::fuzz_target_lib::WitnessValue) value
//! Types of mutations applied:
//! 1. Random (randomly select a new witness value)
//! 2. Max value
//! 3. Min value

use crate::fuzz_lib::fuzz_target_lib::{FieldRepresentation, WitnessValue};
use crate::mutations::configuration::{
    BASIC_WITNESS_MUTATION_CONFIGURATION, WitnessMutationOptions,
};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait WitnessMutator {
    fn mutate(rng: &mut StdRng, value: &mut WitnessValue);
}

/// Return new random witness value
struct RandomMutation;
impl WitnessMutator for RandomMutation {
    fn mutate(rng: &mut StdRng, value: &mut WitnessValue) {
        let mut bytes = [0u8; 17];
        rng.fill(&mut bytes);
        *value = Unstructured::new(&bytes).arbitrary().unwrap();
    }
}

/// Return witness value with max value
struct MaxValueMutation;
impl WitnessMutator for MaxValueMutation {
    fn mutate(_rng: &mut StdRng, value: &mut WitnessValue) {
        let mutated_value = match value {
            WitnessValue::Field(_) => WitnessValue::Field(FieldRepresentation {
                high: 64323764613183177041862057485226039389,
                low: 53438638232309528389504892708671455232, // high * 2^128 + low = p - 1
            }),
            WitnessValue::U64(_) => WitnessValue::U64(u64::MAX),
            WitnessValue::Boolean(_) => WitnessValue::Boolean(true),
            WitnessValue::I64(_) => WitnessValue::I64((1 << 63) - 1), // 2^63 - 1, sign bit is 0
            WitnessValue::I32(_) => WitnessValue::I32((1 << 31) - 1), // 2^31 - 1, sign bit is 0
        };
        *value = mutated_value;
    }
}

/// Return witness value with min value
struct MinValueMutation;
impl WitnessMutator for MinValueMutation {
    fn mutate(_rng: &mut StdRng, value: &mut WitnessValue) {
        let mutated_value = match value {
            WitnessValue::Field(_) => WitnessValue::Field(FieldRepresentation { high: 0, low: 0 }),
            WitnessValue::U64(_) => WitnessValue::U64(0),
            WitnessValue::Boolean(_) => WitnessValue::Boolean(false),
            WitnessValue::I64(_) => WitnessValue::I64(1 << 63), // 2^63, sign bit is 1
            WitnessValue::I32(_) => WitnessValue::I32(1 << 31), // 2^31, sign bit is 1
        };
        *value = mutated_value;
    }
}

/// Add or subtract small value to/from witness value
struct WitnessSmallAddSubMutation;
impl WitnessMutator for WitnessSmallAddSubMutation {
    fn mutate(rng: &mut StdRng, value: &mut WitnessValue) {
        let small_update: i128 = rng.gen_range(0..256);
        *value = match value {
            WitnessValue::Field(field) => WitnessValue::Field(FieldRepresentation {
                high: field.high,
                low: field.low.wrapping_add(small_update as u128),
            }),
            WitnessValue::U64(u64) => WitnessValue::U64(u64.wrapping_add(small_update as u64)),
            WitnessValue::I64(i64) => WitnessValue::I64(i64.wrapping_add(small_update as u64)),
            WitnessValue::I32(i32) => WitnessValue::I32(i32.wrapping_add(small_update as u32)),
            WitnessValue::Boolean(bool) => WitnessValue::Boolean(*bool ^ (small_update % 2 == 1)),
        }
    }
}

/// Add or subtract power of two to/from witness value
struct WitnessAddSubPowerOfTwoMutation;
impl WitnessMutator for WitnessAddSubPowerOfTwoMutation {
    fn mutate(rng: &mut StdRng, value: &mut WitnessValue) {
        let exponent = rng.gen_range(0..254);
        let sign: i128 = if rng.gen_range(0..2) == 0 { 1 } else { -1 };
        *value = match value {
            WitnessValue::Field(field) => {
                // I don't think implementing field addition is worth the effort, so we just
                // add the power of two to the high or low part of the field
                let is_high = exponent > 127;
                let power_of_two: i128 = 1 << (exponent % 128);

                WitnessValue::Field(FieldRepresentation {
                    high: field.high.wrapping_add((is_high as i128 * sign * power_of_two) as u128),
                    low: field.low.wrapping_add((!is_high as i128 * sign * power_of_two) as u128),
                })
            }
            WitnessValue::U64(u64) => {
                WitnessValue::U64(u64.wrapping_add(1 << (exponent % 64) * sign))
            }
            WitnessValue::I64(i64) => {
                WitnessValue::I64(i64.wrapping_add(1 << (exponent % 64) * sign))
            }
            WitnessValue::I32(i32) => {
                WitnessValue::I32(i32.wrapping_add(1 << (exponent % 32) * sign))
            }
            WitnessValue::Boolean(bool) => {
                WitnessValue::Boolean(*bool ^ (1 << (exponent % 2) == 1))
            }
        }
    }
}

pub(crate) fn witness_mutate(witness_value: &mut WitnessValue, rng: &mut StdRng) {
    match BASIC_WITNESS_MUTATION_CONFIGURATION.select(rng) {
        WitnessMutationOptions::Random => RandomMutation::mutate(rng, witness_value),
        WitnessMutationOptions::MaxValue => MaxValueMutation::mutate(rng, witness_value),
        WitnessMutationOptions::MinValue => MinValueMutation::mutate(rng, witness_value),
        WitnessMutationOptions::SmallAddSub => {
            WitnessSmallAddSubMutation::mutate(rng, witness_value)
        }
        WitnessMutationOptions::PowerOfTwoAddSub => {
            WitnessAddSubPowerOfTwoMutation::mutate(rng, witness_value)
        }
    }
}
