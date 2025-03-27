//! This file contains mechanisms for mutating integer InputValues.
//! If the value is a boolean, it just picks a new random value.
//! Otherwise, it performs one of the following mutations:
//! 1. Substitution with one of the fixed values (the selection of values is dependent on the width and signedness of the integer)
//! 2. Substitution with a dictionary value of appropriate width
//! 3. Negation (wrapping for unsigned values)
//! 4. Shift (left or right)
//! 5. Addition, subtraction, "xor", "or" or "and" with a random small value
//! 6. Addition, subtraction, "xor", "or" or "and" with a dictionary value
use std::ops::{BitAnd, BitOr, BitXor, Neg, Shl, Shr};

use acvm::{AcirField, FieldElement};
use noirc_abi::{Sign, input_parser::InputValue};
use num_traits::{
    AsPrimitive, PrimInt, WrappingAdd, WrappingNeg, WrappingSub, Zero,
    ops::overflowing::{OverflowingAdd, OverflowingSub},
};

use rand::{Rng, seq::SliceRandom};
use rand_xorshift::XorShiftRng;

use super::{
    configurations::{
        BASIC_BINARY_INT_OPERATION_MUTATION_CONFIGURATION,
        BASIC_FIXED_INT_SUBSTITUTION_CONFIGURATION, BASIC_INT_TOP_LEVEL_MUTATION_CONFIGURATION,
        BinaryIntOperationMutationOptions, FixedIntSubstitutionOptions, IntTopLevelMutationOptions,
    },
    dictionary::IntDictionary,
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

// Inform compiler that all i{8,16,32,64} have the overflowing_neg method (rust is stupid)
overflowing_neg_impl!(OverflowingNeg, overflowing_neg, i8);
overflowing_neg_impl!(OverflowingNeg, overflowing_neg, i16);
overflowing_neg_impl!(OverflowingNeg, overflowing_neg, i32);
overflowing_neg_impl!(OverflowingNeg, overflowing_neg, i64);
overflowing_neg_impl!(OverflowingNeg, overflowing_neg, i128);
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

// Inform compiler that all integer types have BITS (rust is stupid)
has_bits_impl!(i8);
has_bits_impl!(i16);
has_bits_impl!(i32);
has_bits_impl!(i64);
has_bits_impl!(i128);
has_bits_impl!(u8);
has_bits_impl!(u16);
has_bits_impl!(u32);
has_bits_impl!(u64);
has_bits_impl!(u128);

const MAX_WIDTH: usize = 128;
const NUM_SIGNED_FIXED_VALUES: usize = MAX_WIDTH * 2;
const NUM_UNSIGNED_FIXED_VALUES: usize = NUM_SIGNED_FIXED_VALUES * 2;
const FIXED_SIGNED_VALUES: [i128; NUM_SIGNED_FIXED_VALUES] = calculate_fixed_values_for_signed();
const FIXED_UNSIGNED_VALUES: [u128; NUM_UNSIGNED_FIXED_VALUES] =
    calculate_fixed_values_for_unsigned();

/// Calculate all minimum values for integers with widths from 1 to 64
const fn calculate_all_min_signed() -> [i128; MAX_WIDTH] {
    let mut all_values: [i128; MAX_WIDTH] = [0i128; MAX_WIDTH];
    let mut i = 0;
    while i < MAX_WIDTH {
        all_values[i] = get_min_signed(i + 1);
        i += 1;
    }
    all_values
}

/// Calculate all maximum values for integers with widths from 1 to 64
const fn calculate_all_max_signed() -> [i128; MAX_WIDTH] {
    let mut all_values: [i128; MAX_WIDTH] = [0i128; MAX_WIDTH];
    let mut i = 0;
    while i < MAX_WIDTH {
        all_values[i] = get_max_signed(i + 1);
        i += 1;
    }
    all_values
}

/// Calculate all minimum and maximum values for integers with widths from 1 to 64
const fn calculate_fixed_values_for_signed() -> [i128; NUM_SIGNED_FIXED_VALUES] {
    let mut all_values: [i128; NUM_SIGNED_FIXED_VALUES] = [0i128; NUM_SIGNED_FIXED_VALUES];
    let minimums = calculate_all_min_signed();
    let maximums = calculate_all_max_signed();
    let mut i = 0;
    while i < MAX_WIDTH {
        all_values[i] = minimums[i];
        all_values[i + MAX_WIDTH] = maximums[i];
        i += 1;
    }
    all_values
}

/// Calculate interesting values for unsigned integers
/// - 2^i
/// - 2^i - 1
/// - 2^{64} - 2^i
/// - 2^{64} - (2^i - 1)
const fn calculate_fixed_values_for_unsigned() -> [u128; NUM_UNSIGNED_FIXED_VALUES] {
    let mut all_values: [u128; NUM_UNSIGNED_FIXED_VALUES] = [0u128; NUM_UNSIGNED_FIXED_VALUES];
    let mut i = 0;
    while i < MAX_WIDTH {
        all_values[4 * i] = 2u128.pow(i as u32);
        all_values[4 * i + 1] = 2u128.pow(i as u32) - 1;
        all_values[4 * i + 2] = all_values[4 * i].wrapping_neg();
        all_values[4 * i + 3] = 2u128.pow(64).wrapping_sub(all_values[4 * i + 1]);
        i += 1;
    }
    all_values
}

/// Compute the maximum value of a signed integer of given width
const fn get_max_signed(width: usize) -> i128 {
    if width < MAX_WIDTH { (1i128 << (width - 1)) - 1 } else { i128::MAX }
}

/// Compute the minimum value of a signed integer of given width
const fn get_min_signed(width: usize) -> i128 {
    if width < MAX_WIDTH { -(1i128 << (width - 1)) } else { i128::MIN }
}

/// Convert i128 to given signed integer type, negate and convert to field
fn neg_as_to_field<T: OverflowingNeg + PrimInt + HasBits + AsPrimitive<i128>>(
    input: &i128,
) -> FieldElement {
    let converted: T = T::from(*input).expect("Primitive should convert");
    let (negated, overflowed) = converted.overflowing_neg();
    let negated =
        if overflowed { T::from(0).expect("Should convert a primitive type") } else { negated };
    i128_to_field(negated.as_(), T::BITS)
}

/// Convert u128 to given unsigned integer type, negate and convert to field
fn wrapping_neg_as_to_field<T: WrappingNeg + PrimInt + HasBits + AsPrimitive<u128>>(
    input: &u128,
) -> FieldElement {
    let converted: T = T::from(*input).expect("Primitive should convert");
    u128_to_field(converted.wrapping_neg().as_())
}

/// Convert i128 to given signed integer type, add a small signed value and convert to field
fn add_small<T: OverflowingAdd + PrimInt + HasBits + AsPrimitive<i128>>(
    input: &i128,
    update: i8,
) -> FieldElement {
    let converted: T = T::from(*input).expect("Primitive should convert");
    let update_t: T = T::from(update as i128).expect("Primitive should convert");
    let (after_update, _) = converted.overflowing_add(&update_t);
    i128_to_field(after_update.as_(), T::BITS)
}

/// Convert u128 to given unsigned integer type, add a small unsigned value and convert to field
fn wrapping_add_small_unsigned<T: WrappingAdd + PrimInt + HasBits + AsPrimitive<u128>>(
    input: &u128,
    update: u8,
) -> FieldElement {
    let converted: T = T::from(*input).expect("Primitive should convert");
    let update_t: T = T::from(update as u128).expect("Primitive should convert");
    let after_update = converted.wrapping_add(&update_t);
    u128_to_field(after_update.as_())
}

/// Convert u128 to given unsigned integer type, subtract a small unsigned value and convert to field
fn wrapping_sub_small_unsigned<T: WrappingSub + PrimInt + HasBits + AsPrimitive<u128>>(
    input: &u128,
    update: u8,
) -> FieldElement {
    let converted: T = T::from(*input).expect("Primitive should convert");
    let update_t: T = T::from(update as u128).expect("Primitive should convert");
    let after_update = converted.wrapping_sub(&update_t);
    u128_to_field(after_update.as_())
}

/// Perform a signed integer binary operation on 2 field elements according to `mutation_operation`. Get field as a result
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
    mutation_operation: BinaryIntOperationMutationOptions,
) -> FieldElement {
    let width = T::BITS;
    let lhs_int = T::from(field_to_i128(lhs, width)).expect("Should convert");
    let rhs_int = T::from(field_to_i128(rhs, width)).expect("Should convert");
    let result_int = match mutation_operation {
        BinaryIntOperationMutationOptions::Add => lhs_int.overflowing_add(&rhs_int).0,
        BinaryIntOperationMutationOptions::Sub => lhs_int.overflowing_sub(&rhs_int).0,
        BinaryIntOperationMutationOptions::Xor => lhs_int ^ rhs_int,
        BinaryIntOperationMutationOptions::And => lhs_int & rhs_int,
        BinaryIntOperationMutationOptions::Or => lhs_int | rhs_int,
    };
    i128_to_field(result_int.as_(), width)
}

/// Perform an integer binary operation on 2 field elements according to `mutation_operation`. Get field as a result
fn add_sub_pow_2_update<
    T: WrappingAdd
        + WrappingSub
        + HasBits
        + BitXor<Output = T>
        + BitAnd<Output = T>
        + BitOr<Output = T>
        + AsPrimitive<u128>
        + PrimInt,
>(
    lhs: &FieldElement,
    prng: &mut XorShiftRng,
) -> FieldElement {
    let width = T::BITS;
    let lhs_int = T::from(field_to_u128(lhs)).expect("Should convert");
    let update = T::from(1u128 << prng.gen_range(0..width)).expect("Should convert");
    let result_int = if prng.gen_range(0..2).is_zero() {
        lhs_int.wrapping_add(&update)
    } else {
        lhs_int.wrapping_sub(&update)
    };
    u128_to_field(result_int.as_())
}

/// Perform an integer binary operation on 2 field elements according to `mutation_operation`. Get field as a result
fn add_sub_pow_2_update_signed<
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
    prng: &mut XorShiftRng,
) -> FieldElement {
    let width = T::BITS;
    let lhs_int = T::from(field_to_i128(lhs, width)).expect("Should convert");
    let update = T::from(1i128 << prng.gen_range(0..(width - 1))).expect("Should convert");
    let result_int = if prng.gen_range(0..2).is_zero() {
        lhs_int.wrapping_add(&update)
    } else {
        lhs_int.wrapping_sub(&update)
    };
    i128_to_field(result_int.as_(), width)
}
/// Perform an unsigned integer binary operation on 2 field elements according to `mutation_operation`. Get field as a result
fn add_sub_xor_and_or_unsigned<
    T: WrappingAdd
        + WrappingSub
        + HasBits
        + BitXor<Output = T>
        + BitAnd<Output = T>
        + BitOr<Output = T>
        + AsPrimitive<u128>
        + PrimInt,
>(
    lhs: &FieldElement,
    rhs: &FieldElement,
    mutation_operation: BinaryIntOperationMutationOptions,
) -> FieldElement {
    let lhs_int = T::from(field_to_u128(lhs)).expect("Should convert");
    let rhs_int = T::from(field_to_u128(rhs)).expect("Should convert");
    let result_int = match mutation_operation {
        BinaryIntOperationMutationOptions::Add => lhs_int.wrapping_add(&rhs_int),
        BinaryIntOperationMutationOptions::Sub => lhs_int.wrapping_sub(&rhs_int),
        BinaryIntOperationMutationOptions::Xor => lhs_int ^ rhs_int,
        BinaryIntOperationMutationOptions::And => lhs_int & rhs_int,
        BinaryIntOperationMutationOptions::Or => lhs_int | rhs_int,
    };
    u128_to_field(result_int.as_())
}

/// Perform a shift operation on the value and convert to field
/// The lowest bit of direction_and_magnitude represents shift direction, higher bits - shift value
fn shift_signed_int<
    T: Shl<u32, Output = T> + Shr<u32, Output = T> + PrimInt + HasBits + AsPrimitive<i128>,
>(
    input: &i128,
    direction_and_magnitude: u32,
) -> FieldElement {
    let width = T::BITS;
    let converted: T = T::from(*input).expect("Primitive should convert");
    let after_update = if (direction_and_magnitude & 1u32).is_zero() {
        converted >> (direction_and_magnitude >> 1)
    } else {
        converted << (direction_and_magnitude >> 1)
    };
    i128_to_field(after_update.as_(), width)
}

/// Perform a shift operation on the value and convert to field
/// The lowest bit of direction_and_magnitude represents shift direction, higher bits - shift value
fn shift_unsigned_int<
    T: Shl<u32, Output = T> + Shr<u32, Output = T> + PrimInt + HasBits + AsPrimitive<u128>,
>(
    input: &u128,
    direction_and_magnitude: u32,
) -> FieldElement {
    let converted: T = T::from(*input).expect("Primitive should convert");
    let after_update = if (direction_and_magnitude & 1u32).is_zero() {
        converted >> (direction_and_magnitude >> 1)
    } else {
        converted << (direction_and_magnitude >> 1)
    };
    u128_to_field(after_update.as_())
}

/// Convert a value from field representation to integer representation
/// Width is needed to detect negative values in 2-complement form
fn field_to_i128(input: &FieldElement, width: u32) -> i128 {
    let mut initial_i128 = input.to_i128();
    if initial_i128 >= 2i128.pow(width - 1) {
        initial_i128 -= 2i128.pow(width);
    }
    initial_i128
}
/// Convert a value from field representation to integer representation
fn field_to_u128(input: &FieldElement) -> u128 {
    input.to_u128()
}

/// Convert a value from integer representation to field representation
/// Width is needed to detect negative values in 2-complement form
fn i128_to_field(value: i128, width: u32) -> FieldElement {
    let mut new_value = value;
    if value < 0 {
        new_value += 2i128.pow(width);
    }
    FieldElement::from(new_value)
}

/// Convert a value from integer representation to field representation
fn u128_to_field(value: u128) -> FieldElement {
    FieldElement::from(value)
}

/// Generate a random unsigned integer of given width and convert to field
fn generate_random_for_width(prng: &mut XorShiftRng, width: u32) -> FieldElement {
    FieldElement::from(prng.gen_range(0..(1i128 << width)))
}

struct IntMutator<'a> {
    dictionary: &'a IntDictionary,
    prng: &'a mut XorShiftRng,
}

impl<'a> IntMutator<'a> {
    pub fn new(dictionary: &'a IntDictionary, prng: &'a mut XorShiftRng) -> Self {
        Self { dictionary, prng }
    }

    /// Get one of the fixed values in place of the original value
    fn substitute_signed_int_with_fixed_value(&mut self, width: u32) -> InputValue {
        let value = match BASIC_FIXED_INT_SUBSTITUTION_CONFIGURATION.select(self.prng) {
            FixedIntSubstitutionOptions::Minimum => {
                FIXED_SIGNED_VALUES[self.prng.gen_range(0..width as usize)]
            }
            FixedIntSubstitutionOptions::Maximum => {
                FIXED_SIGNED_VALUES[self.prng.gen_range(64..(64 + width) as usize)]
            }
            FixedIntSubstitutionOptions::Pow2 => 2i128.pow(self.prng.gen_range(0..(width - 1))),
        };
        let checked_value = match width {
            8 => value.clamp(i8::MIN as i128, i8::MAX as i128),
            16 => value.clamp(i16::MIN as i128, i16::MAX as i128),
            32 => value.clamp(i32::MIN as i128, i32::MAX as i128),
            64 => value.clamp(i64::MIN as i128, i64::MAX as i128),
            128 => value,
            _ => {
                panic!("Shouldn't be reachable")
            }
        };

        InputValue::Field(i128_to_field(checked_value, width))
    }

    /// Negate a signed value
    fn negate_signed_int(&mut self, input: &i128, width: u32) -> InputValue {
        InputValue::Field(match width {
            8 => neg_as_to_field::<i8>(input),
            16 => neg_as_to_field::<i16>(input),
            32 => neg_as_to_field::<i32>(input),
            64 => neg_as_to_field::<i64>(input),
            128 => neg_as_to_field::<i128>(input),
            _ => {
                panic!("Shouldn't be reachable")
            }
        })
    }

    /// Add or subtract a small signed value to input
    fn sub_add_small_value_signed(&mut self, input: &i128, width: u32) -> InputValue {
        let update = self.prng.gen_range(i8::MIN..=i8::MAX);
        InputValue::Field(match width {
            8 => add_small::<i8>(input, update),
            16 => add_small::<i16>(input, update),
            32 => add_small::<i32>(input, update),
            64 => add_small::<i64>(input, update),
            128 => add_small::<i128>(input, update),
            _ => {
                panic!("Shouldn't be reachable")
            }
        })
    }

    /// Shift signed value
    fn shift_signed_int(&mut self, input: &i128, width: u32) -> InputValue {
        let exponent = self.prng.gen_range(1..=(width * 2 - 1));
        InputValue::Field(match width {
            8 => shift_signed_int::<i8>(input, exponent),
            16 => shift_signed_int::<i16>(input, exponent),
            32 => shift_signed_int::<i32>(input, exponent),
            64 => shift_signed_int::<i64>(input, exponent),
            128 => shift_signed_int::<i128>(input, exponent),
            _ => {
                panic!("Shouldn't be reachable")
            }
        })
    }

    /// Substitute with a dictionary value
    fn substitute_with_dictionary_value(&mut self, width: u32) -> InputValue {
        let width_dictionary = self.dictionary.get_dictionary_by_width(width);
        if width_dictionary.is_empty() {
            return InputValue::Field(generate_random_for_width(self.prng, width));
        }
        InputValue::Field(*width_dictionary.choose(self.prng).unwrap())
    }

    /// Perform a binary operation with the given value and a random dictionary value
    fn perform_signed_binary_operation_with_dictionary(
        &mut self,
        input: &FieldElement,
        width: u32,
    ) -> InputValue {
        let width_dictionary = self.dictionary.get_dictionary_by_width(width);
        if width_dictionary.is_empty() {
            return InputValue::Field(generate_random_for_width(self.prng, width));
        }
        let dictionary_value = width_dictionary.choose(self.prng).unwrap();
        let chosen_operation = BASIC_BINARY_INT_OPERATION_MUTATION_CONFIGURATION.select(self.prng);
        InputValue::Field(match width {
            8 => add_sub_xor_and_or_signed::<i8>(input, dictionary_value, chosen_operation),
            16 => add_sub_xor_and_or_signed::<i16>(input, dictionary_value, chosen_operation),
            32 => add_sub_xor_and_or_signed::<i32>(input, dictionary_value, chosen_operation),
            64 => add_sub_xor_and_or_signed::<i64>(input, dictionary_value, chosen_operation),
            128 => add_sub_xor_and_or_signed::<i128>(input, dictionary_value, chosen_operation),
            _ => {
                panic!("Shouldn't be reachable")
            }
        })
    }

    fn perform_pow_2_update_signed(&mut self, input: &FieldElement, width: u32) -> InputValue {
        InputValue::Field(match width {
            8 => add_sub_pow_2_update_signed::<i8>(input, self.prng),
            16 => add_sub_pow_2_update_signed::<i16>(input, self.prng),
            32 => add_sub_pow_2_update_signed::<i32>(input, self.prng),
            64 => add_sub_pow_2_update_signed::<i64>(input, self.prng),
            128 => add_sub_pow_2_update_signed::<i128>(input, self.prng),
            _ => {
                panic!("Shouldn't be reachable")
            }
        })
    }

    fn perform_pow_2_update_unsigned(&mut self, input: &FieldElement, width: u32) -> InputValue {
        InputValue::Field(match width {
            8 => add_sub_pow_2_update::<u8>(input, self.prng),
            16 => add_sub_pow_2_update::<u16>(input, self.prng),
            32 => add_sub_pow_2_update::<u32>(input, self.prng),
            64 => add_sub_pow_2_update::<u64>(input, self.prng),
            128 => add_sub_pow_2_update::<u128>(input, self.prng),
            _ => {
                panic!("Shouldn't be reachable")
            }
        })
    }
    /// Perform a mutation on a signed int
    fn mutate_signed(&mut self, input: &FieldElement, width: u32) -> InputValue {
        let initial_i128 = field_to_i128(input, width);
        let mutation_type = BASIC_INT_TOP_LEVEL_MUTATION_CONFIGURATION.select(self.prng);
        match mutation_type {
            IntTopLevelMutationOptions::FixedSubstitution => {
                self.substitute_signed_int_with_fixed_value(width)
            }
            IntTopLevelMutationOptions::DictionarySubstitution => {
                self.substitute_with_dictionary_value(width)
            }
            IntTopLevelMutationOptions::Negation => self.negate_signed_int(&initial_i128, width),
            IntTopLevelMutationOptions::Shift => self.shift_signed_int(&initial_i128, width),
            IntTopLevelMutationOptions::SmallValueUpdate => {
                self.sub_add_small_value_signed(&initial_i128, width)
            }
            IntTopLevelMutationOptions::DictionaryValueUpdate => {
                self.perform_signed_binary_operation_with_dictionary(input, width)
            }
            IntTopLevelMutationOptions::Pow2Update => {
                self.perform_pow_2_update_signed(input, width)
            }
        }
    }

    // Get one of the fixed values in place of the original value
    fn substitute_unsigned_int_with_fixed_value(&mut self, width: u32) -> InputValue {
        InputValue::Field(FieldElement::from(
            FIXED_UNSIGNED_VALUES[self.prng.gen_range(0..(width * 4) as usize)]
                & (u128::MAX >> (u128::BITS - width)),
        ))
    }

    /// Negate an unsigned value
    fn negate_unsigned_int(&mut self, input: &u128, width: u32) -> InputValue {
        InputValue::Field(match width {
            8 => wrapping_neg_as_to_field::<u8>(input),
            16 => wrapping_neg_as_to_field::<u16>(input),
            32 => wrapping_neg_as_to_field::<u32>(input),
            64 => wrapping_neg_as_to_field::<u64>(input),
            128 => wrapping_neg_as_to_field::<u128>(input),
            _ => {
                panic!("Shouldn't be reachable")
            }
        })
    }

    /// Shift signed value
    fn shift_unsigned_int(&mut self, input: &u128, width: u32) -> InputValue {
        let exponent = self.prng.gen_range(1..=(width * 2 - 1));
        InputValue::Field(match width {
            8 => shift_unsigned_int::<u8>(input, exponent),
            16 => shift_unsigned_int::<u16>(input, exponent),
            32 => shift_unsigned_int::<u32>(input, exponent),
            64 => shift_unsigned_int::<u64>(input, exponent),
            128 => shift_unsigned_int::<u128>(input, exponent),
            _ => {
                panic!("Shouldn't be reachable")
            }
        })
    }

    /// Add or subtract a small signed value to input
    fn sub_add_small_value_unsigned(&mut self, input: &u128, width: u32) -> InputValue {
        let update = self.prng.gen_range(u8::MIN..=u8::MAX);
        let is_add = self.prng.gen_range(0..2).is_zero();

        // Probability of addition and subtraction is equal
        if is_add {
            InputValue::Field(match width {
                8 => wrapping_add_small_unsigned::<u8>(input, update),
                16 => wrapping_add_small_unsigned::<u16>(input, update),
                32 => wrapping_add_small_unsigned::<u32>(input, update),
                64 => wrapping_add_small_unsigned::<u64>(input, update),
                128 => wrapping_add_small_unsigned::<u128>(input, update),
                _ => {
                    panic!("Shouldn't be reachable")
                }
            })
        } else {
            InputValue::Field(match width {
                8 => wrapping_sub_small_unsigned::<u8>(input, update),
                16 => wrapping_sub_small_unsigned::<u16>(input, update),
                32 => wrapping_sub_small_unsigned::<u32>(input, update),
                64 => wrapping_sub_small_unsigned::<u64>(input, update),
                128 => wrapping_sub_small_unsigned::<u128>(input, update),
                _ => {
                    panic!("Shouldn't be reachable")
                }
            })
        }
    }

    /// Perform a binary operation with the given value and a random dictionary value
    fn perform_unsigned_binary_operation_with_dictionary(
        &mut self,
        input: &FieldElement,
        width: u32,
    ) -> InputValue {
        let width_dictionary = self.dictionary.get_dictionary_by_width(width);
        if width_dictionary.is_empty() {
            return InputValue::Field(generate_random_for_width(self.prng, width));
        }
        let dictionary_value = width_dictionary.choose(self.prng).unwrap();
        let chosen_operation = BASIC_BINARY_INT_OPERATION_MUTATION_CONFIGURATION.select(self.prng);
        InputValue::Field(match width {
            8 => add_sub_xor_and_or_unsigned::<u8>(input, dictionary_value, chosen_operation),
            16 => add_sub_xor_and_or_unsigned::<u16>(input, dictionary_value, chosen_operation),
            32 => add_sub_xor_and_or_unsigned::<u32>(input, dictionary_value, chosen_operation),
            64 => add_sub_xor_and_or_unsigned::<u64>(input, dictionary_value, chosen_operation),
            128 => add_sub_xor_and_or_unsigned::<u128>(input, dictionary_value, chosen_operation),
            _ => {
                panic!("Shouldn't be reachable")
            }
        })
    }
    /// Perform a mutation on an unsigned int
    pub fn mutate_unsigned(&mut self, input: &FieldElement, width: u32) -> InputValue {
        let initial_u128 = field_to_u128(input);
        match BASIC_INT_TOP_LEVEL_MUTATION_CONFIGURATION.select(self.prng) {
            IntTopLevelMutationOptions::FixedSubstitution => {
                self.substitute_unsigned_int_with_fixed_value(width)
            }
            IntTopLevelMutationOptions::DictionarySubstitution => {
                self.substitute_with_dictionary_value(width)
            }
            IntTopLevelMutationOptions::Negation => self.negate_unsigned_int(&initial_u128, width),
            IntTopLevelMutationOptions::Shift => self.shift_unsigned_int(&initial_u128, width),
            IntTopLevelMutationOptions::SmallValueUpdate => {
                self.sub_add_small_value_unsigned(&initial_u128, width)
            }
            IntTopLevelMutationOptions::DictionaryValueUpdate => {
                self.perform_unsigned_binary_operation_with_dictionary(input, width)
            }
            IntTopLevelMutationOptions::Pow2Update => {
                self.perform_pow_2_update_unsigned(input, width)
            }
        }
    }

    /// Mutate an input value depending on the sign and width
    pub fn mutate(&mut self, input: &InputValue, sign: &Sign, width: u32) -> InputValue {
        let initial_field_value = match input {
            InputValue::Field(inner_field) => inner_field,
            _ => panic!("Shouldn't be used with other input value types"),
        };
        assert!(
            width == 1 || width == 8 || width == 16 || width == 32 || width == 64 || width == 128
        );

        // If it's just one bit just get a random value
        if width == 1 {
            assert!(*sign == Sign::Unsigned);
            InputValue::Field(FieldElement::from(self.prng.gen_range(0..1u32)))
        } else {
            match sign {
                Sign::Signed => self.mutate_signed(initial_field_value, width),
                Sign::Unsigned => self.mutate_unsigned(initial_field_value, width),
            }
        }
    }
}

/// Mutate an integer input value
pub fn mutate_int_input_value(
    previous_input: &InputValue,
    sign: &Sign,
    width: u32,
    dictionary: &IntDictionary,
    prng: &mut XorShiftRng,
) -> InputValue {
    let mut int_mutator = IntMutator::new(dictionary, prng);
    int_mutator.mutate(previous_input, sign, width)
}
