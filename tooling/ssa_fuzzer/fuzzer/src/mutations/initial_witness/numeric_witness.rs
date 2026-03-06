//! This file contains mechanisms for deterministically mutating a given [WitnessValue](crate::fuzz_lib::fuzz_target_lib::WitnessValue) value
//! Types of mutations applied:
//! 1. Random (randomly select a new witness value)
//! 2. Max value
//! 3. Min value
//! 4. Small add/sub
//! 5. Power of two add/sub

use crate::fuzz_lib::initial_witness::{FieldRepresentation, WitnessValueNumeric};
use crate::mutations::configuration::{
    NumericWitnessMutationConfig, SIZE_OF_SMALL_ARBITRARY_BUFFER, WitnessMutationOptions,
};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

pub(crate) fn generate_element_of_the_same_type(
    rng: &mut StdRng,
    value: WitnessValueNumeric,
) -> WitnessValueNumeric {
    let mut bytes = [0u8; SIZE_OF_SMALL_ARBITRARY_BUFFER];
    rng.fill(&mut bytes);
    match value {
        WitnessValueNumeric::Field(_) => {
            WitnessValueNumeric::Field(Unstructured::new(&bytes).arbitrary().unwrap())
        }
        WitnessValueNumeric::U128(_) => {
            WitnessValueNumeric::U128(Unstructured::new(&bytes).arbitrary().unwrap())
        }
        WitnessValueNumeric::U64(_) => {
            WitnessValueNumeric::U64(Unstructured::new(&bytes).arbitrary().unwrap())
        }
        WitnessValueNumeric::U32(_) => {
            WitnessValueNumeric::U32(Unstructured::new(&bytes).arbitrary().unwrap())
        }
        WitnessValueNumeric::U16(_) => {
            WitnessValueNumeric::U16(Unstructured::new(&bytes).arbitrary().unwrap())
        }
        WitnessValueNumeric::U8(_) => {
            WitnessValueNumeric::U8(Unstructured::new(&bytes).arbitrary().unwrap())
        }
        WitnessValueNumeric::Boolean(_) => {
            WitnessValueNumeric::Boolean(Unstructured::new(&bytes).arbitrary().unwrap())
        }
        WitnessValueNumeric::I64(_) => {
            WitnessValueNumeric::I64(Unstructured::new(&bytes).arbitrary().unwrap())
        }
        WitnessValueNumeric::I32(_) => {
            WitnessValueNumeric::I32(Unstructured::new(&bytes).arbitrary().unwrap())
        }
        WitnessValueNumeric::I16(_) => {
            WitnessValueNumeric::I16(Unstructured::new(&bytes).arbitrary().unwrap())
        }
        WitnessValueNumeric::I8(_) => {
            WitnessValueNumeric::I8(Unstructured::new(&bytes).arbitrary().unwrap())
        }
    }
}

/// Return new random witness value
struct RandomMutation;
impl RandomMutation {
    fn mutate(rng: &mut StdRng, value: &mut WitnessValueNumeric) {
        let mut bytes = [0u8; 17];
        rng.fill(&mut bytes);
        *value = Unstructured::new(&bytes).arbitrary().unwrap();
    }
}

/// Return witness value with max value
struct MaxValueMutation;
impl MaxValueMutation {
    fn mutate(value: &mut WitnessValueNumeric) {
        let mutated_value = match value {
            WitnessValueNumeric::Field(_) => WitnessValueNumeric::Field(FieldRepresentation {
                high: 64323764613183177041862057485226039389,
                low: 53438638232309528389504892708671455232, // high * 2^128 + low = p - 1
            }),
            WitnessValueNumeric::U128(_) => WitnessValueNumeric::U128(u128::MAX),
            WitnessValueNumeric::U64(_) => WitnessValueNumeric::U64(u64::MAX),
            WitnessValueNumeric::U32(_) => WitnessValueNumeric::U32(u32::MAX),
            WitnessValueNumeric::U16(_) => WitnessValueNumeric::U16(u16::MAX),
            WitnessValueNumeric::U8(_) => WitnessValueNumeric::U8(u8::MAX),
            WitnessValueNumeric::Boolean(_) => WitnessValueNumeric::Boolean(true),
            WitnessValueNumeric::I64(_) => WitnessValueNumeric::I64((1 << 63) - 1), // 2^63 - 1, sign bit is 0
            WitnessValueNumeric::I32(_) => WitnessValueNumeric::I32((1 << 31) - 1), // 2^31 - 1, sign bit is 0
            WitnessValueNumeric::I16(_) => WitnessValueNumeric::I16((1 << 15) - 1), // 2^15 - 1, sign bit is 0
            WitnessValueNumeric::I8(_) => WitnessValueNumeric::I8((1 << 7) - 1), // 2^7 - 1, sign bit is 0
        };
        *value = mutated_value;
    }
}

/// Return witness value with min value
struct MinValueMutation;
impl MinValueMutation {
    fn mutate(value: &mut WitnessValueNumeric) {
        let mutated_value = match value {
            WitnessValueNumeric::Field(_) => {
                WitnessValueNumeric::Field(FieldRepresentation { high: 0, low: 0 })
            }
            WitnessValueNumeric::U128(_) => WitnessValueNumeric::U128(u128::MIN),
            WitnessValueNumeric::U64(_) => WitnessValueNumeric::U64(u64::MIN),
            WitnessValueNumeric::U32(_) => WitnessValueNumeric::U32(u32::MIN),
            WitnessValueNumeric::U16(_) => WitnessValueNumeric::U16(u16::MIN),
            WitnessValueNumeric::U8(_) => WitnessValueNumeric::U8(u8::MIN),
            WitnessValueNumeric::Boolean(_) => WitnessValueNumeric::Boolean(false),
            WitnessValueNumeric::I64(_) => WitnessValueNumeric::I64(1 << 63), // 2^63, sign bit is 1
            WitnessValueNumeric::I32(_) => WitnessValueNumeric::I32(1 << 31), // 2^31, sign bit is 1
            WitnessValueNumeric::I16(_) => WitnessValueNumeric::I16(1 << 15), // 2^15, sign bit is 1
            WitnessValueNumeric::I8(_) => WitnessValueNumeric::I8(1 << 7),    // 2^7, sign bit is 1
        };
        *value = mutated_value;
    }
}

/// Add or subtract small value to/from witness value
struct WitnessSmallAddSubMutation;
impl WitnessSmallAddSubMutation {
    fn mutate(rng: &mut StdRng, value: &mut WitnessValueNumeric) {
        let small_update: i128 = rng.random_range(0..256);
        let sign: bool = rng.random_range(0..2) == 0;
        *value = match value {
            WitnessValueNumeric::Field(field) => WitnessValueNumeric::Field(FieldRepresentation {
                high: field.high,
                low: if !sign {
                    field.low.wrapping_add(small_update as u128)
                } else {
                    field.low.wrapping_sub(small_update as u128)
                },
            }),
            WitnessValueNumeric::U128(u128) => WitnessValueNumeric::U128(if !sign {
                u128.wrapping_add(small_update as u128)
            } else {
                u128.wrapping_sub(small_update as u128)
            }),
            WitnessValueNumeric::U32(u32) => WitnessValueNumeric::U32(if !sign {
                u32.wrapping_add(small_update as u32)
            } else {
                u32.wrapping_sub(small_update as u32)
            }),
            WitnessValueNumeric::U16(u16) => WitnessValueNumeric::U16(if !sign {
                u16.wrapping_add(small_update as u16)
            } else {
                u16.wrapping_sub(small_update as u16)
            }),
            WitnessValueNumeric::U8(u8) => WitnessValueNumeric::U8(if !sign {
                u8.wrapping_add(small_update as u8)
            } else {
                u8.wrapping_sub(small_update as u8)
            }),
            WitnessValueNumeric::U64(u64) => WitnessValueNumeric::U64(if !sign {
                u64.wrapping_add(small_update as u64)
            } else {
                u64.wrapping_sub(small_update as u64)
            }),
            WitnessValueNumeric::I64(i64) => WitnessValueNumeric::I64(if !sign {
                i64.wrapping_add(small_update as u64)
            } else {
                i64.wrapping_sub(small_update as u64)
            }),
            WitnessValueNumeric::I32(i32) => WitnessValueNumeric::I32(if !sign {
                i32.wrapping_add(small_update as u32)
            } else {
                i32.wrapping_sub(small_update as u32)
            }),
            WitnessValueNumeric::I16(i16) => WitnessValueNumeric::I16(if !sign {
                i16.wrapping_add(small_update as u16)
            } else {
                i16.wrapping_sub(small_update as u16)
            }),
            WitnessValueNumeric::I8(i8) => WitnessValueNumeric::I8(if !sign {
                i8.wrapping_add(small_update as u8)
            } else {
                i8.wrapping_sub(small_update as u8)
            }),
            WitnessValueNumeric::Boolean(bool) => {
                WitnessValueNumeric::Boolean(*bool ^ (small_update % 2 == 1))
            }
        }
    }
}

/// Add or subtract power of two to/from witness value
struct WitnessAddSubPowerOfTwoMutation;
impl WitnessAddSubPowerOfTwoMutation {
    fn mutate(rng: &mut StdRng, value: &mut WitnessValueNumeric) {
        let exponent = rng.random_range(0..254);
        let sign: bool = rng.random_range(0..2) == 0;
        *value = match value {
            WitnessValueNumeric::Field(field) => {
                // I don't think implementing field addition is worth the effort, so we just
                // add the power of two to the high or low part of the field
                let is_high = exponent > 127;
                let power_of_two: i128 = 1 << (exponent % 128);

                WitnessValueNumeric::Field(FieldRepresentation {
                    high: if !sign {
                        field.high.wrapping_add((i128::from(is_high) * power_of_two) as u128)
                    } else {
                        field.high.wrapping_sub((i128::from(is_high) * power_of_two) as u128)
                    },
                    low: if !sign {
                        field.low.wrapping_add((i128::from(!is_high) * power_of_two) as u128)
                    } else {
                        field.low.wrapping_sub((i128::from(!is_high) * power_of_two) as u128)
                    },
                })
            }
            WitnessValueNumeric::U128(u128) => WitnessValueNumeric::U128(if !sign {
                u128.wrapping_add(1 << (exponent % 128))
            } else {
                u128.wrapping_sub(1 << (exponent % 128))
            }),
            WitnessValueNumeric::U32(u32) => WitnessValueNumeric::U32(if !sign {
                u32.wrapping_add(1 << (exponent % 32))
            } else {
                u32.wrapping_sub(1 << (exponent % 32))
            }),
            WitnessValueNumeric::U16(u16) => WitnessValueNumeric::U16(if !sign {
                u16.wrapping_add(1 << (exponent % 16))
            } else {
                u16.wrapping_sub(1 << (exponent % 16))
            }),
            WitnessValueNumeric::U8(u8) => WitnessValueNumeric::U8(if !sign {
                u8.wrapping_add(1 << (exponent % 8))
            } else {
                u8.wrapping_sub(1 << (exponent % 8))
            }),
            WitnessValueNumeric::U64(u64) => WitnessValueNumeric::U64(if !sign {
                u64.wrapping_add(1 << (exponent % 64))
            } else {
                u64.wrapping_sub(1 << (exponent % 64))
            }),
            WitnessValueNumeric::I64(i64) => WitnessValueNumeric::I64(if !sign {
                i64.wrapping_add(1 << (exponent % 64))
            } else {
                i64.wrapping_sub(1 << (exponent % 64))
            }),
            WitnessValueNumeric::I32(i32) => WitnessValueNumeric::I32(if !sign {
                i32.wrapping_add(1 << (exponent % 32))
            } else {
                i32.wrapping_sub(1 << (exponent % 32))
            }),
            WitnessValueNumeric::I16(i16) => WitnessValueNumeric::I16(if !sign {
                i16.wrapping_add(1 << (exponent % 16))
            } else {
                i16.wrapping_sub(1 << (exponent % 16))
            }),
            WitnessValueNumeric::I8(i8) => WitnessValueNumeric::I8(if !sign {
                i8.wrapping_add(1 << (exponent % 8))
            } else {
                i8.wrapping_sub(1 << (exponent % 8))
            }),
            WitnessValueNumeric::Boolean(bool) => {
                WitnessValueNumeric::Boolean(*bool ^ (1 << (exponent % 2) == 1))
            }
        }
    }
}

pub(crate) fn mutate(
    witness_value: &mut WitnessValueNumeric,
    rng: &mut StdRng,
    config: NumericWitnessMutationConfig,
) {
    match config.select(rng) {
        WitnessMutationOptions::Random => RandomMutation::mutate(rng, witness_value),
        WitnessMutationOptions::MaxValue => MaxValueMutation::mutate(witness_value),
        WitnessMutationOptions::MinValue => MinValueMutation::mutate(witness_value),
        WitnessMutationOptions::SmallAddSub => {
            WitnessSmallAddSubMutation::mutate(rng, witness_value);
        }
        WitnessMutationOptions::PowerOfTwoAddSub => {
            WitnessAddSubPowerOfTwoMutation::mutate(rng, witness_value);
        }
    }
}
