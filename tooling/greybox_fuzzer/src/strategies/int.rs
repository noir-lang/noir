use std::ops::{BitAnd, BitOr, BitXor, Neg, Shl, Shr};

use acvm::{AcirField, FieldElement};
use noirc_abi::{input_parser::InputValue, Sign};
use num_traits::{
    ops::overflowing::{OverflowingAdd, OverflowingSub},
    AsPrimitive, PrimInt, WrappingAdd, WrappingNeg, WrappingSub, Zero,
};
/// Performs negation with a flag for overflow.
trait OverflowingNeg: Sized + Neg {
    /// Returns a tuple of the negated value along with a boolean indicating whether an arithmetic overflow would occur.
    /// If an overflow would have occurred then the wrapped value is returned.
    fn overflowing_neg(&self) -> (Self, bool);
}
macro_rules! overflowing_neg_impl {
    ($trait_name:ident, $method:ident, $t:ty) => {
        impl $trait_name for $t {
            #[inline]
            fn $method(&self) -> (Self, bool) {
                <$t>::$method(*self)
            }
        }
    };
}

overflowing_neg_impl!(OverflowingNeg, overflowing_neg, i8);
overflowing_neg_impl!(OverflowingNeg, overflowing_neg, i16);
overflowing_neg_impl!(OverflowingNeg, overflowing_neg, i32);
overflowing_neg_impl!(OverflowingNeg, overflowing_neg, i64);

pub trait HasBits {
    const BITS: u32;
}
macro_rules! has_bits_impl {
    ( $t:ty) => {
        impl HasBits for $t {
            const BITS: u32 = <$t>::BITS;
        }
    };
}

has_bits_impl!(i8);
has_bits_impl!(i16);
has_bits_impl!(i32);
has_bits_impl!(i64);
has_bits_impl!(u8);
has_bits_impl!(u16);
has_bits_impl!(u32);
has_bits_impl!(u64);

use rand::{distributions::Standard, prelude::Distribution, seq::SliceRandom, Rng};
use rand_xorshift::XorShiftRng;

const NUM_FIXED_VALUES: usize = 128;
const FIXED_SIGNED_VALUES: [i128; NUM_FIXED_VALUES] = calculate_fixed_values_for_signed();
const FIXED_UNSIGNED_VALUES: [i128; 256] = calculate_fixed_values_for_unsigned();
const SUBSTITUTE_BY_FIXED_WEIGHT: usize = 0x20;
const NEGATE_WEIGHT: usize = 0x2;
const SHIFT_WEIGHT: usize = 0x20;
const SUB_ADD_SMALL_VALUE_WEIGHT: usize = 0x80;
const SUBSTITUTE_DICTIONARY_VALUE_WEIGHT: usize = 0x30;
const SUB_ADD_AND_XOR_OR_DICTIONARY_VALUE_WEIGHT: usize = 0x30;
const SIGNED_TOTAL_WEIGHT: usize = SUBSTITUTE_BY_FIXED_WEIGHT
    + NEGATE_WEIGHT
    + SHIFT_WEIGHT
    + SUB_ADD_SMALL_VALUE_WEIGHT
    + SUBSTITUTE_DICTIONARY_VALUE_WEIGHT
    + SUB_ADD_AND_XOR_OR_DICTIONARY_VALUE_WEIGHT;
const UNSIGNED_SIGNED_TOTAL_WEIGHT: usize = SUBSTITUTE_BY_FIXED_WEIGHT
    + NEGATE_WEIGHT
    + SHIFT_WEIGHT
    + SUB_ADD_SMALL_VALUE_WEIGHT
    + SUBSTITUTE_DICTIONARY_VALUE_WEIGHT
    + SUB_ADD_AND_XOR_OR_DICTIONARY_VALUE_WEIGHT;

pub struct IntDictionary {
    width_dictionaries: [Vec<FieldElement>; 4],
}

impl IntDictionary {
    pub fn new(original_dictionary: &Vec<FieldElement>) -> Self {
        Self { width_dictionaries: filter_dictionary_by_width(original_dictionary) }
    }
    pub fn get_dictionary_by_width(&self, width: u32) -> &Vec<FieldElement> {
        match width {
            8 => &self.width_dictionaries[0],
            16 => &self.width_dictionaries[1],
            32 => &self.width_dictionaries[2],
            64 => &self.width_dictionaries[3],
            _ => panic!("Only widths 8, 16, 32, 64 are supported"),
        }
    }
}

fn filter_dictionary_by_width(original_dictionary: &Vec<FieldElement>) -> [Vec<FieldElement>; 4] {
    let mut width8_dict = Vec::new();
    let mut width16_dict = Vec::new();
    let mut width32_dict = Vec::new();
    let mut width64_dict = Vec::new();
    const MAX_U8: i128 = 2i128.pow(8) - 1;
    const MAX_U16: i128 = 2i128.pow(16) - 1;
    const MAX_U32: i128 = 2i128.pow(32) - 1;
    const MAX_U64: i128 = 2i128.pow(64) - 1;
    for element in original_dictionary.iter().copied() {
        let el_i128 = element.to_i128();
        if el_i128 <= 0 {
            continue;
        }
        if el_i128 < MAX_U64 {
            width64_dict.push(element);
        }
        if el_i128 < MAX_U32 {
            width32_dict.push(element);
        }
        if el_i128 < MAX_U16 {
            width16_dict.push(element);
        }
        if el_i128 < MAX_U8 {
            width8_dict.push(element);
        }
    }
    [width8_dict, width16_dict, width32_dict, width64_dict]
}

const fn calculate_all_min_signed() -> [i128; 64] {
    let mut all_values: [i128; 64] = [0i128; 64];
    let mut i = 0;
    while i < 64 {
        all_values[i] = get_min_signed(i + 1);
        i += 1;
    }
    all_values
}
const fn calculate_all_max_signed() -> [i128; 64] {
    let mut all_values: [i128; 64] = [0i128; 64];
    let mut i = 0;
    while i < 64 {
        all_values[i] = get_max_signed(i + 1);
        i += 1;
    }
    all_values
}
const fn calculate_fixed_values_for_signed() -> [i128; 128] {
    let mut all_values: [i128; 128] = [0i128; 128];
    let minimums = calculate_all_min_signed();
    let maximums = calculate_all_max_signed();
    let mut i = 0;
    while i < 64 {
        all_values[i] = minimums[i];
        all_values[i + 64] = maximums[i];
        i += 1;
    }
    all_values
}
const fn calculate_fixed_values_for_unsigned() -> [i128; 256] {
    let mut all_values: [i128; 256] = [0i128; 256];
    let mut i = 0;
    while i < 64 {
        all_values[4 * i as usize] = 2i128.pow(i);
        all_values[(4 * i + 1) as usize] = 2i128.pow(i) - 1;
        all_values[(4 * i + 2) as usize] = 2i128.pow(64) - all_values[(4 * i) as usize];
        all_values[(4 * i + 3) as usize] = 2i128.pow(64) - all_values[(4 * i + 1) as usize];
        i += 1;
    }
    all_values
}
const fn get_max_signed(width: usize) -> i128 {
    if width < 128 {
        (1i128 << (width - 1)) - 1
    } else {
        i128::MAX
    }
}

const fn get_min_signed(width: usize) -> i128 {
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

fn wrapping_neg_as_to_field<T: WrappingNeg + PrimInt + HasBits + AsPrimitive<i128>>(
    input: i128,
) -> FieldElement {
    let converted: T = T::from(input).expect("Primitive should convert");
    i128_to_field(converted.wrapping_neg().as_(), T::BITS)
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

fn wrapping_add_small_unsigned<T: WrappingAdd + PrimInt + HasBits + AsPrimitive<i128>>(
    input: i128,
    update: u8,
) -> FieldElement {
    let converted: T = T::from(input).expect("Primitive should convert");
    let update_t: T = T::from(update as i128).expect("Primitive should convert");
    let after_update = converted.wrapping_add(&update_t);
    i128_to_field(after_update.as_(), T::BITS)
}

fn wrapping_sub_small_unsigned<T: WrappingSub + PrimInt + HasBits + AsPrimitive<i128>>(
    input: i128,
    update: u8,
) -> FieldElement {
    let converted: T = T::from(input).expect("Primitive should convert");
    let update_t: T = T::from(update as i128).expect("Primitive should convert");
    let after_update = converted.wrapping_sub(&update_t);
    i128_to_field(after_update.as_(), T::BITS)
}
enum MutationOperation {
    Add,
    Sub,
    Xor,
    And,
    Or,
}

impl Distribution<MutationOperation> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> MutationOperation {
        match rng.gen_range(0..=4) {
            // rand 0.8
            0 => MutationOperation::Add,
            1 => MutationOperation::Sub,
            2 => MutationOperation::Xor,
            3 => MutationOperation::And,
            4 => MutationOperation::Or,
            _ => panic!("Shouldn't be reachable"),
        }
    }
}
fn add_sub_xor_and_or_signed<
    T: OverflowingAdd
        + OverflowingSub
        + HasBits
        + BitXor<Output = T>
        + BitAnd<Output = T>
        + BitOr<Output = T>
        + AsPrimitive<i128>
        + PrimInt,
>(
    lhs: &FieldElement,
    rhs: &FieldElement,
    mutation_operation: MutationOperation,
) -> FieldElement {
    let width = T::BITS;
    let lhs_int = T::from(field_to_i128(*lhs, width)).expect("Should convert");
    let rhs_int = T::from(field_to_i128(*rhs, width)).expect("Should convert");
    let result_int = match mutation_operation {
        MutationOperation::Add => lhs_int.overflowing_add(&rhs_int).0,
        MutationOperation::Sub => lhs_int.overflowing_sub(&rhs_int).0,
        MutationOperation::Xor => lhs_int ^ rhs_int,
        MutationOperation::And => lhs_int & rhs_int,
        MutationOperation::Or => lhs_int | rhs_int,
    };
    i128_to_field(result_int.as_(), width)
}

fn add_sub_xor_and_or_unsigned<
    T: WrappingAdd
        + WrappingSub
        + HasBits
        + BitXor<Output = T>
        + BitAnd<Output = T>
        + BitOr<Output = T>
        + AsPrimitive<i128>
        + PrimInt,
>(
    lhs: &FieldElement,
    rhs: &FieldElement,
    mutation_operation: MutationOperation,
) -> FieldElement {
    let width = T::BITS;
    let lhs_int = T::from(field_to_i128(*lhs, width + 1)).expect("Should convert");
    let rhs_int = T::from(field_to_i128(*rhs, width + 1)).expect("Should convert");
    let result_int = match mutation_operation {
        MutationOperation::Add => lhs_int.wrapping_add(&rhs_int),
        MutationOperation::Sub => lhs_int.wrapping_sub(&rhs_int),
        MutationOperation::Xor => lhs_int ^ rhs_int,
        MutationOperation::And => lhs_int & rhs_int,
        MutationOperation::Or => lhs_int | rhs_int,
    };
    i128_to_field(result_int.as_(), width)
}
fn shift<T: Shl<u32, Output = T> + Shr<u32, Output = T> + PrimInt + HasBits + AsPrimitive<i128>>(
    input: i128,
    exponent: u32,
) -> FieldElement {
    let width = T::BITS;
    let converted: T = T::from(input).expect("Primitive should convert");
    let after_update = if (exponent & 1u32).is_zero() {
        converted >> (exponent >> 1)
    } else {
        converted << (exponent >> 1)
    };
    i128_to_field(after_update.as_(), width)
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

fn generate_random_for_width(prng: &mut XorShiftRng, width: u32) -> FieldElement {
    FieldElement::from(prng.gen_range(0..(2i128.pow(width))))
}
pub fn mutate_int_input_value(
    previous_input: &InputValue,
    sign: &Sign,
    width: u32,
    dictionary: &IntDictionary,
    prng: &mut XorShiftRng,
) -> InputValue {
    let initial_field_value = match previous_input {
        InputValue::Field(inner_field) => inner_field,
        _ => panic!("Shouldn't be used with other input value types"),
    }
    .clone();
    assert!(width == 1 || width == 8 || width == 16 || width == 32 || width == 64);

    // If it's just one bit just get a random value
    if width == 1 {
        assert!(*sign == Sign::Unsigned);
        return InputValue::Field(FieldElement::from(prng.gen_range(0..1u32)));
    } else {
        match sign {
            Sign::Signed => {
                let initial_i128 = field_to_i128(initial_field_value, width);
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
                        _ => {
                            panic!("Shouldn't be reachable")
                        }
                    };
                }
                selector -= SUBSTITUTE_BY_FIXED_WEIGHT;

                if selector < NEGATE_WEIGHT {
                    return InputValue::Field(match width {
                        8 => neg_as_to_field::<i8>(initial_i128),
                        16 => neg_as_to_field::<i16>(initial_i128),
                        32 => neg_as_to_field::<i32>(initial_i128),
                        64 => neg_as_to_field::<i64>(initial_i128),
                        _ => {
                            panic!("Shouldn't be reachable")
                        }
                    });
                }
                selector -= NEGATE_WEIGHT;
                if selector < SHIFT_WEIGHT {
                    let exponent = prng.gen_range(1..=(width * 2 - 1));
                    return InputValue::Field(match width {
                        8 => shift::<i8>(initial_i128, exponent),
                        16 => shift::<i16>(initial_i128, exponent),
                        32 => shift::<i32>(initial_i128, exponent),
                        64 => shift::<i64>(initial_i128, exponent),
                        _ => {
                            panic!("Shouldn't be reachable")
                        }
                    });
                }
                selector -= SHIFT_WEIGHT;
                if selector < SUB_ADD_SMALL_VALUE_WEIGHT {
                    let update = prng.gen_range(i8::min_value()..=i8::max_value());
                    return InputValue::Field(match width {
                        8 => add_small::<i8>(initial_i128, update),
                        16 => add_small::<i16>(initial_i128, update),
                        32 => add_small::<i32>(initial_i128, update),
                        64 => add_small::<i64>(initial_i128, update),
                        _ => {
                            panic!("Shouldn't be reachable")
                        }
                    });
                }
                selector -= SUB_ADD_SMALL_VALUE_WEIGHT;
                if selector < SUBSTITUTE_DICTIONARY_VALUE_WEIGHT {
                    let width_dictionary = dictionary.get_dictionary_by_width(width);
                    if width_dictionary.is_empty() {
                        return InputValue::Field(generate_random_for_width(prng, width));
                    }
                    return InputValue::Field(width_dictionary.choose(prng).unwrap().clone());
                }
                selector -= SUBSTITUTE_DICTIONARY_VALUE_WEIGHT;
                assert!(selector < SUB_ADD_AND_XOR_OR_DICTIONARY_VALUE_WEIGHT);
                let width_dictionary = dictionary.get_dictionary_by_width(width);
                if width_dictionary.is_empty() {
                    return InputValue::Field(generate_random_for_width(prng, width));
                }
                let dictionary_value = width_dictionary.choose(prng).unwrap();
                let chosen_operation = prng.gen::<MutationOperation>();
                return InputValue::Field(match width {
                    8 => add_sub_xor_and_or_signed::<i8>(
                        &initial_field_value,
                        &dictionary_value,
                        chosen_operation,
                    ),
                    16 => add_sub_xor_and_or_signed::<i16>(
                        &initial_field_value,
                        &dictionary_value,
                        chosen_operation,
                    ),
                    32 => add_sub_xor_and_or_signed::<i32>(
                        &initial_field_value,
                        &dictionary_value,
                        chosen_operation,
                    ),
                    64 => add_sub_xor_and_or_signed::<i64>(
                        &initial_field_value,
                        &dictionary_value,
                        chosen_operation,
                    ),
                    _ => {
                        panic!("Shouldn't be reachable")
                    }
                });
            }
            Sign::Unsigned => {
                let initial_i128 = initial_field_value.to_i128();
                let mut selector = prng.gen_range(0..UNSIGNED_SIGNED_TOTAL_WEIGHT);
                if selector < SUBSTITUTE_BY_FIXED_WEIGHT {
                    return InputValue::Field(FieldElement::from(
                        FIXED_UNSIGNED_VALUES[prng.gen_range(0..(width * 4) as usize)],
                    ));
                }
                selector -= SUBSTITUTE_BY_FIXED_WEIGHT;

                if selector < NEGATE_WEIGHT {
                    return InputValue::Field(match width {
                        8 => wrapping_neg_as_to_field::<u8>(initial_i128),
                        16 => wrapping_neg_as_to_field::<u16>(initial_i128),
                        32 => wrapping_neg_as_to_field::<u32>(initial_i128),
                        64 => wrapping_neg_as_to_field::<u64>(initial_i128),
                        _ => {
                            panic!("Shouldn't be reachable")
                        }
                    });
                }
                selector -= NEGATE_WEIGHT;
                if selector < SHIFT_WEIGHT {
                    let exponent = prng.gen_range(1..=(width * 2 - 1));
                    return InputValue::Field(match width {
                        8 => shift::<u8>(initial_i128, exponent),
                        16 => shift::<u16>(initial_i128, exponent),
                        32 => shift::<u32>(initial_i128, exponent),
                        64 => shift::<u64>(initial_i128, exponent),
                        _ => {
                            panic!("Shouldn't be reachable")
                        }
                    });
                }
                selector -= SHIFT_WEIGHT;
                if selector < SUB_ADD_SMALL_VALUE_WEIGHT {
                    let update = prng.gen_range(u8::min_value()..=u8::max_value());
                    if prng.gen_range(0..2) == 0 {
                        return InputValue::Field(match width {
                            8 => wrapping_add_small_unsigned::<u8>(initial_i128, update),
                            16 => wrapping_add_small_unsigned::<u16>(initial_i128, update),
                            32 => wrapping_add_small_unsigned::<u32>(initial_i128, update),
                            64 => wrapping_add_small_unsigned::<u64>(initial_i128, update),
                            _ => {
                                panic!("Shouldn't be reachable")
                            }
                        });
                    } else {
                        return InputValue::Field(match width {
                            8 => wrapping_sub_small_unsigned::<u8>(initial_i128, update),
                            16 => wrapping_sub_small_unsigned::<u16>(initial_i128, update),
                            32 => wrapping_sub_small_unsigned::<u32>(initial_i128, update),
                            64 => wrapping_sub_small_unsigned::<u64>(initial_i128, update),
                            _ => {
                                panic!("Shouldn't be reachable")
                            }
                        });
                    }
                }
                selector -= SUB_ADD_SMALL_VALUE_WEIGHT;
                if selector < SUBSTITUTE_DICTIONARY_VALUE_WEIGHT {
                    let width_dictionary = dictionary.get_dictionary_by_width(width);
                    if width_dictionary.is_empty() {
                        return InputValue::Field(generate_random_for_width(prng, width));
                    }
                    return InputValue::Field(width_dictionary.choose(prng).unwrap().clone());
                }
                selector -= SUBSTITUTE_DICTIONARY_VALUE_WEIGHT;
                assert!(selector < SUB_ADD_AND_XOR_OR_DICTIONARY_VALUE_WEIGHT);
                let width_dictionary = dictionary.get_dictionary_by_width(width);
                if width_dictionary.is_empty() {
                    return InputValue::Field(generate_random_for_width(prng, width));
                }
                let dictionary_value = width_dictionary.choose(prng).unwrap();
                let chosen_operation = prng.gen::<MutationOperation>();
                return InputValue::Field(match width {
                    8 => add_sub_xor_and_or_unsigned::<i8>(
                        &initial_field_value,
                        &dictionary_value,
                        chosen_operation,
                    ),
                    16 => add_sub_xor_and_or_unsigned::<i16>(
                        &initial_field_value,
                        &dictionary_value,
                        chosen_operation,
                    ),
                    32 => add_sub_xor_and_or_unsigned::<i32>(
                        &initial_field_value,
                        &dictionary_value,
                        chosen_operation,
                    ),
                    64 => add_sub_xor_and_or_unsigned::<i64>(
                        &initial_field_value,
                        &dictionary_value,
                        chosen_operation,
                    ),
                    _ => {
                        panic!("Shouldn't be reachable")
                    }
                });
            }
        }
    }
}
