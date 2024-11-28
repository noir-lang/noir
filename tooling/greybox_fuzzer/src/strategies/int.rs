use std::ops::{Add, Neg};

use acvm::{AcirField, FieldElement};
use noir_fuzzer::dictionary;
use noirc_abi::{input_parser::InputValue, Sign};
use num_traits::{
    ops::overflowing::{OverflowingAdd, OverflowingMul},
    AsPrimitive, Num, Pow, PrimInt,
};
/// Performs negation with a flag for overflow.
trait OverflowingNeg: Sized + Neg {
    /// Returns a tuple of the negated value along with a boolean indicating whether an arithmetic overflow would occur.
    /// If an overflow would have occurred then the wrapped value is returned.
    fn overflowing_neg(&self) -> (Self, bool);
}

trait HasBits {
    const BITS: u32;
}
impl OverflowingNeg for i8 {
    fn overflowing_neg(&self) -> (Self, bool) {
        self.overflowing_neg()
    }
}
impl OverflowingNeg for i16 {
    fn overflowing_neg(&self) -> (Self, bool) {
        self.overflowing_neg()
    }
}
impl OverflowingNeg for i32 {
    fn overflowing_neg(&self) -> (Self, bool) {
        self.overflowing_neg()
    }
}
impl OverflowingNeg for i64 {
    fn overflowing_neg(&self) -> (Self, bool) {
        self.overflowing_neg()
    }
}

use rand::Rng;
use rand_xorshift::XorShiftRng;
const NUM_FIXED_VALUES: usize = 128;
const FIXED_SIGNED_VALUES: [i128; NUM_FIXED_VALUES] = calculate_fixed_value_for_signed();
const SUBSTITUTE_BY_FIXED_WEIGHT: usize = 0x20;
const NEGATE_WEIGHT: usize = 0x2;
const MUL_BY_POW2_WEIGHT: usize = 0x20;
const SUB_ADD_SMALL_VALUE_WEIGHT: usize = 0x80;
const SIGNED_TOTAL_WEIGHT: usize =
    SUBSTITUTE_BY_FIXED_WEIGHT + NEGATE_WEIGHT + MUL_BY_POW2_WEIGHT + SUB_ADD_SMALL_VALUE_WEIGHT;
const UNSIGNED_SIGNED_TOTAL_WEIGHT: usize = 0;

fn calculate_all_min_signed() -> [i128; 64] {
    let mut all_values: [i128; 64];
    for i in 0..64 {
        all_values[i] = get_min_signed(i + 1);
    }
    all_values
}
fn calculate_all_max_signed() -> [i128; 64] {
    let mut all_values: [i128; 64];
    for i in 0..64 {
        all_values[i] = get_max_signed(i + 1);
    }
    all_values
}
fn calculate_fixed_value_for_signed() -> [i128; 128] {
    let mut all_values: [i128; 128];
    let minimums = calculate_all_min_signed();
    let maximums = calculate_all_max_signed();
    for i in 0..64 {
        all_values[i] = minimums[i];
        all_values[i + 64] = maximums[i];
    }
    all_values
}
fn get_max_signed(width: usize) -> i128 {
    if width < 128 {
        (1i128 << (width - 1)) - 1
    } else {
        i128::MAX
    }
}

fn get_min_signed(width: usize) -> i128 {
    if width < 128 {
        -(1i128 << (width - 1))
    } else {
        i128::MIN
    }
}
fn neg_as_to_field<T: OverflowingNeg + PrimInt + HasBits + AsPrimitive<i128>>(
    input: i128,
) -> FieldElement {
    let converted: T = T::from(input).expect("Primitive should convert");
    let (negated, overflowed) = converted.overflowing_neg();
    let negated =
        if overflowed { T::from(0).expect("Should convert a primitive type") } else { negated };
    i128_to_field(negated.as_(), T::BITS)
}

fn add_small<T: OverflowingAdd + PrimInt + HasBits + AsPrimitive<i128>>(
    input: i128,
    update: i8,
) -> FieldElement {
    let converted: T = T::from(input).expect("Primitive should convert");
    let update_t: T = T::from(update as i128).expect("Primitive should convert");
    let (after_update, _) = converted.overflowing_add(&update_t);
    i128_to_field(after_update.as_(), T::BITS)
}
fn mul_by_pow2<T: OverflowingMul + PrimInt + HasBits + AsPrimitive<i128>>(
    input: i128,
    exponent: u32,
) -> FieldElement {
    let converted: T = T::from(input).expect("Primitive should convert");
    let (after_update, _) =
        converted.overflowing_mul(&T::from(2).expect("Primitive should convert").pow(exponent));
    i128_to_field(after_update.as_(), T::BITS)
}

fn field_to_i128(input: FieldElement, width: u32) -> i128 {
    let mut initial_i128 = input.to_i128();
    if initial_i128 >= 2i128.pow(width - 1) {
        initial_i128 -= 2i128.pow(width);
    }
    initial_i128
}

fn i128_to_field(value: i128, width: u32) -> FieldElement {
    let mut new_value = value;
    if value < 0 {
        new_value += 2i128.pow(width);
    }
    FieldElement::from(new_value)
}
pub fn mutate_int_input_value(
    previous_input: &InputValue,
    sign: &Sign,
    width: u32,
    dictionary: &Vec<FieldElement>,
    prng: &mut XorShiftRng,
) -> InputValue {
    let initial_field_value = match previous_input {
        InputValue::Field(inner_field) => inner_field,
        _ => panic!("Shouldn't be used with other input value types"),
    }
    .clone();
    assert!(!dictionary.is_empty());
    assert!(width == 1 || width == 8 || width == 16 || width == 32 || width == 64);

    // If it's just one bit just get a random value
    if (width == 1) {
        assert!(*sign == Sign::Unsigned);
        return InputValue::Field(FieldElement::from(prng.gen_range(0..1u32)));
    } else {
        match sign {
            Sign::Signed => {
                let mut initial_i128 = field_to_i128(initial_field_value, width);
                let mut selector = prng.gen_range(0..SIGNED_TOTAL_WEIGHT);
                if selector < SUBSTITUTE_BY_FIXED_WEIGHT {
                    return match prng.gen_range(0..3) {
                        0 => {
                            // Minimum values
                            InputValue::Field(i128_to_field(
                                FIXED_SIGNED_VALUES[prng.gen_range(0..width as usize)],
                                width,
                            ))
                        }
                        1 => {
                            // Maximum values
                            InputValue::Field(i128_to_field(
                                FIXED_SIGNED_VALUES[prng.gen_range(64..(64 + width) as usize)],
                                width,
                            ))
                        }
                        2 => {
                            // Powers of two
                            InputValue::Field(i128_to_field(
                                2i128.pow(prng.gen_range(0..width)),
                                width,
                            ))
                        }
                    };
                }
                selector -= SUBSTITUTE_BY_FIXED_WEIGHT;

                if selector < NEGATE_WEIGHT {
                    return match width {
                        8 => InputValue::Field(neg_as_to_field::<i8>(initial_i128)),
                        16 => InputValue::Field(neg_as_to_field::<i16>(initial_i128)),
                        32 => InputValue::Field(neg_as_to_field::<i32>(initial_i128)),
                        64 => InputValue::Field(neg_as_to_field::<i64>(initial_i128)),
                    };
                }
                selector -= NEGATE_WEIGHT;
                if selector < MUL_BY_POW2_WEIGHT {
                    let exponent = prng.gen_range(1i128..=(width - 1) as i128);
                    return match width {
                        8 => mul_by_pow2::<i8>(initial_i128, exponent),
                        16 => mul_by_pow2::<i16>(initial_i128, exponent),
                        32 => mul_by_pow2::<i32>(initial_i128, exponent),
                        64 => mul_by_pow2::<i64>(initial_i128, exponent),
                    };
                }
                selector -= MUL_BY_POW2_WEIGHT;
                if selector < SUB_ADD_SMALL_VALUE_WEIGHT {
                    let update = prng.gen_range(i8::min_value()..=i8::max_value());
                    return match width {
                        8 => add_small::<i8>(initial_i128, update),
                        16 => add_small::<i16>(initial_i128, update),
                        32 => add_small::<i32>(initial_i128, update),
                        64 => add_small::<i64>(initial_i128, update),
                    };
                }
                selector -= SUB_ADD_SMALL_VALUE_WEIGHT;
            }
            Sign::Unsigned => {
                let initial_u64 = initial_field_value
                    .try_to_u64()
                    .expect("Maximum width is 64, why would it fail?");
            }
        }
    }
    ()
    // Types of mutations:
    // 1. Substitution (0, 1, -1, powers of 2, powers of 2 minus one)
    // 2. Negation/inversion
    // 3. Multiplication by a power of 2 (division by a power of 2)
    // 4. Addition/subtraction of a small value
    // 5. Additions/subtraction of a power of two
    // 6. Addition/Substitution of a Value from the dictionary
}
