use std::ops::{BitAnd, BitOr, BitXor, Neg, Shl, Shr};

use acvm::{AcirField, FieldElement};
use noirc_abi::{input_parser::InputValue, Sign};
use num_traits::{
    ops::overflowing::{OverflowingAdd, OverflowingSub},
    AsPrimitive, PrimInt, WrappingAdd, WrappingNeg, WrappingSub, Zero,
};

use rand::{seq::SliceRandom, Rng};
use rand_xorshift::XorShiftRng;
/// This file contains mechanisms for mutating integer InputValues.
/// If the value is a boolean, it just picks a new random value.
/// Otherwise, it performs one of the following mutations:
/// 1. Substitution with one of the fixed values (the selection of values is dependent on the width and signedness of the integer)
/// 2. Substitution with a dictionary value of appropriate width
/// 3. Negation (wrapping for unsigned values)
/// 4. Shift (left or right)
/// 5. Addition, subtraction, "xor", "or" or "and" with a random small value
/// 6. Addition, subtraction, "xor", "or" or "and" with a dictionary value

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
has_bits_impl!(u8);
has_bits_impl!(u16);
has_bits_impl!(u32);
has_bits_impl!(u64);

/// A dictionary for integer values. Separated by width
#[derive(Default)]
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

/// Filter values in the original dictionary collected from the program into 4 categories, separated by width of integers into which those elements can fit
fn filter_dictionary_by_width(original_dictionary: &Vec<FieldElement>) -> [Vec<FieldElement>; 4] {
    let mut width8_dict = Vec::new();
    let mut width16_dict = Vec::new();
    let mut width32_dict = Vec::new();
    let mut width64_dict = Vec::new();
    const MAX_U8: i128 = u8::MAX as i128;
    const MAX_U16: i128 = u16::MAX as i128;
    const MAX_U32: i128 = u32::MAX as i128;
    const MAX_U64: i128 = u64::MAX as i128;
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

const NUM_FIXED_VALUES: usize = 128;
const FIXED_SIGNED_VALUES: [i128; NUM_FIXED_VALUES] = calculate_fixed_values_for_signed();
const FIXED_UNSIGNED_VALUES: [i128; 256] = calculate_fixed_values_for_unsigned();

/// Calculate all minimum values for integers with widths from 1 to 64
const fn calculate_all_min_signed() -> [i128; 64] {
    let mut all_values: [i128; 64] = [0i128; 64];
    let mut i = 0;
    while i < 64 {
        all_values[i] = get_min_signed(i + 1);
        i += 1;
    }
    all_values
}

/// Calculate all maximum values for integers with widths from 1 to 64
const fn calculate_all_max_signed() -> [i128; 64] {
    let mut all_values: [i128; 64] = [0i128; 64];
    let mut i = 0;
    while i < 64 {
        all_values[i] = get_max_signed(i + 1);
        i += 1;
    }
    all_values
}

/// Calculate all minimum and maximum values for integers with widths from 1 to 64
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

/// Calculate interesting values for unsigned integers
/// - 2^i
/// - 2^i - 1
/// - 2^{64} - 2^i
/// - 2^{64} - (2^i - 1)
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

/// Compute the maximum value of a signed integer of given width
const fn get_max_signed(width: usize) -> i128 {
    if width < 128 {
        (1i128 << (width - 1)) - 1
    } else {
        i128::MAX
    }
}

/// Compute the minimum value of a signed integer of given width
const fn get_min_signed(width: usize) -> i128 {
    if width < 128 {
        -(1i128 << (width - 1))
    } else {
        i128::MIN
    }
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

/// Convert i128 to given unsigned integer type, negate and convert to field
fn wrapping_neg_as_to_field<T: WrappingNeg + PrimInt + HasBits + AsPrimitive<i128>>(
    input: &i128,
) -> FieldElement {
    let converted: T = T::from(*input).expect("Primitive should convert");
    i128_to_field(converted.wrapping_neg().as_(), T::BITS)
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

/// Convert i128 to given unsigned integer type, add a small unsigned value and convert to field
fn wrapping_add_small_unsigned<T: WrappingAdd + PrimInt + HasBits + AsPrimitive<i128>>(
    input: &i128,
    update: u8,
) -> FieldElement {
    let converted: T = T::from(*input).expect("Primitive should convert");
    let update_t: T = T::from(update as i128).expect("Primitive should convert");
    let after_update = converted.wrapping_add(&update_t);
    i128_to_field(after_update.as_(), T::BITS)
}

/// Convert i128 to given unsigned integer type, subtract a small unsigned value and convert to field
fn wrapping_sub_small_unsigned<T: WrappingSub + PrimInt + HasBits + AsPrimitive<i128>>(
    input: &i128,
    update: u8,
) -> FieldElement {
    let converted: T = T::from(*input).expect("Primitive should convert");
    let update_t: T = T::from(update as i128).expect("Primitive should convert");
    let after_update = converted.wrapping_sub(&update_t);
    i128_to_field(after_update.as_(), T::BITS)
}
enum FixedSubstitution {
    Minimum,
    Maximum,
    Pow2,
}

struct FixedSubstitutionConfiguration {
    minimum_weight: usize,
    maximum_weight: usize,
    #[allow(unused)]
    pow2_weight: usize,
    total_weight: usize,
}

impl FixedSubstitutionConfiguration {
    #[allow(unused)]
    pub fn new(minimum_weight: usize, maximum_weight: usize, pow2_weight: usize) -> Self {
        let total_weight = minimum_weight + maximum_weight + pow2_weight;
        Self { minimum_weight, maximum_weight, pow2_weight, total_weight }
    }
    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> FixedSubstitution {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.minimum_weight {
            return FixedSubstitution::Minimum;
        }
        selector -= self.minimum_weight;
        if selector < self.maximum_weight {
            return FixedSubstitution::Maximum;
        }
        return FixedSubstitution::Pow2;
    }
}
const BASIC_FIXED_SUBSTITUTION_CONFIGURATION: FixedSubstitutionConfiguration =
    FixedSubstitutionConfiguration {
        minimum_weight: 1,
        maximum_weight: 1,
        pow2_weight: 1,
        total_weight: 1 + 1 + 1,
    };

enum BinaryOperationMutation {
    Add,
    Sub,
    And,
    Or,
    Xor,
}

struct BinaryOperationMutationConfiguration {
    addition_weight: usize,
    subtraction_weight: usize,
    and_operation_weight: usize,
    or_operation_weight: usize,
    #[allow(unused)]
    xor_operation_weight: usize,
    total_weight: usize,
}

impl BinaryOperationMutationConfiguration {
    #[allow(unused)]
    pub fn new(
        addition_weight: usize,
        subtraction_weight: usize,
        and_operation_weight: usize,
        or_operation_weight: usize,
        xor_operation_weight: usize,
    ) -> Self {
        let total_weight = addition_weight
            + subtraction_weight
            + and_operation_weight
            + or_operation_weight
            + xor_operation_weight;
        Self {
            addition_weight,
            subtraction_weight,
            and_operation_weight,
            or_operation_weight,
            xor_operation_weight,
            total_weight,
        }
    }
    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> BinaryOperationMutation {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.addition_weight {
            return BinaryOperationMutation::Add;
        }
        selector -= self.addition_weight;
        if selector < self.subtraction_weight {
            return BinaryOperationMutation::Sub;
        }
        selector -= self.subtraction_weight;
        if selector < self.and_operation_weight {
            return BinaryOperationMutation::And;
        }
        selector -= self.and_operation_weight;
        if selector < self.or_operation_weight {
            return BinaryOperationMutation::Or;
        }
        return BinaryOperationMutation::Xor;
    }
}
const BASIC_BINARY_OPERATION_MUTATION_CONFIGURATION: BinaryOperationMutationConfiguration =
    BinaryOperationMutationConfiguration {
        addition_weight: 1,
        subtraction_weight: 1,
        and_operation_weight: 1,
        or_operation_weight: 1,
        xor_operation_weight: 1,
        total_weight: 1 + 1 + 1 + 1 + 1,
    };

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
    mutation_operation: BinaryOperationMutation,
) -> FieldElement {
    let width = T::BITS;
    let lhs_int = T::from(field_to_i128(lhs, width)).expect("Should convert");
    let rhs_int = T::from(field_to_i128(rhs, width)).expect("Should convert");
    let result_int = match mutation_operation {
        BinaryOperationMutation::Add => lhs_int.overflowing_add(&rhs_int).0,
        BinaryOperationMutation::Sub => lhs_int.overflowing_sub(&rhs_int).0,
        BinaryOperationMutation::Xor => lhs_int ^ rhs_int,
        BinaryOperationMutation::And => lhs_int & rhs_int,
        BinaryOperationMutation::Or => lhs_int | rhs_int,
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
        + AsPrimitive<i128>
        + PrimInt,
>(
    lhs: &FieldElement,
    rhs: &FieldElement,
    mutation_operation: BinaryOperationMutation,
) -> FieldElement {
    let width = T::BITS;
    let lhs_int = T::from(field_to_i128(lhs, width + 1)).expect("Should convert");
    let rhs_int = T::from(field_to_i128(rhs, width + 1)).expect("Should convert");
    let result_int = match mutation_operation {
        BinaryOperationMutation::Add => lhs_int.wrapping_add(&rhs_int),
        BinaryOperationMutation::Sub => lhs_int.wrapping_sub(&rhs_int),
        BinaryOperationMutation::Xor => lhs_int ^ rhs_int,
        BinaryOperationMutation::And => lhs_int & rhs_int,
        BinaryOperationMutation::Or => lhs_int | rhs_int,
    };
    i128_to_field(result_int.as_(), width)
}

/// Perform a shift operation on the value and convert to field
/// The lowest bit of direction_and_magnitude represents shift direction, higher bits - shift value
fn shift<T: Shl<u32, Output = T> + Shr<u32, Output = T> + PrimInt + HasBits + AsPrimitive<i128>>(
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

/// Convert a value from field representation to integer representation
/// Width is needed to detect negative values in 2-complement form
fn field_to_i128(input: &FieldElement, width: u32) -> i128 {
    let mut initial_i128 = input.to_i128();
    if initial_i128 >= 2i128.pow(width - 1) {
        initial_i128 -= 2i128.pow(width);
    }
    initial_i128
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

/// Generate a random unsigned integer of given width and convert to field
fn generate_random_for_width(prng: &mut XorShiftRng, width: u32) -> FieldElement {
    FieldElement::from(prng.gen_range(0..(2i128 << width)))
}

enum IntTopLevelMutation {
    FixedSubstitution,
    DictionarySubstitution,
    Negation,
    Shift,
    SmallValueUpdate,
    DictionaryValueUpdate,
}

struct IntTopLevelMutationConfiguration {
    fixed_substitution_weight: usize,
    dictionary_substitution_weight: usize,
    negation_weight: usize,
    shift_weight: usize,
    small_value_update_weight: usize,
    #[allow(unused)]
    dictionary_value_update_weight: usize,
    total_weight: usize,
}

impl IntTopLevelMutationConfiguration {
    #[allow(unused)]
    pub fn new(
        fixed_substitution_weight: usize,
        dictionary_substitution_weight: usize,
        negation_weight: usize,
        shift_weight: usize,
        small_value_update_weight: usize,
        dictionary_value_update_weight: usize,
    ) -> Self {
        let total_weight = fixed_substitution_weight
            + dictionary_substitution_weight
            + negation_weight
            + shift_weight
            + small_value_update_weight
            + dictionary_value_update_weight;
        Self {
            fixed_substitution_weight,
            dictionary_substitution_weight,
            negation_weight,
            shift_weight,
            small_value_update_weight,
            dictionary_value_update_weight,
            total_weight,
        }
    }

    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> IntTopLevelMutation {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.fixed_substitution_weight {
            return IntTopLevelMutation::FixedSubstitution;
        }
        selector -= self.fixed_substitution_weight;
        if selector < self.dictionary_substitution_weight {
            return IntTopLevelMutation::DictionarySubstitution;
        }
        selector -= self.dictionary_substitution_weight;
        if selector < self.negation_weight {
            return IntTopLevelMutation::Negation;
        }
        selector -= self.negation_weight;
        if selector < self.shift_weight {
            return IntTopLevelMutation::Shift;
        }
        selector -= self.shift_weight;
        if selector < self.small_value_update_weight {
            return IntTopLevelMutation::SmallValueUpdate;
        }
        return IntTopLevelMutation::DictionaryValueUpdate;
    }
}

const BASIC_INT_TOP_LEVEL_MUTATION_CONFIGURATION: IntTopLevelMutationConfiguration =
    IntTopLevelMutationConfiguration {
        fixed_substitution_weight: 0x20,
        dictionary_substitution_weight: 0x30,
        negation_weight: 0x2,
        shift_weight: 0x8,
        small_value_update_weight: 0x80,
        dictionary_value_update_weight: 0x30,
        total_weight: 0x20 + 0x30 + 0x2 + 0x8 + 0x80 + 0x30,
    };

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
        return InputValue::Field(i128_to_field(
            match BASIC_FIXED_SUBSTITUTION_CONFIGURATION.select(self.prng) {
                FixedSubstitution::Minimum => {
                    FIXED_SIGNED_VALUES[self.prng.gen_range(0..width as usize)]
                }
                FixedSubstitution::Maximum => {
                    FIXED_SIGNED_VALUES[self.prng.gen_range(64..(64 + width) as usize)]
                }
                FixedSubstitution::Pow2 => 2i128.pow(self.prng.gen_range(0..width)),
            },
            width,
        ));
    }

    /// Negate a signed value
    fn negate_signed_int(&mut self, input: &i128, width: u32) -> InputValue {
        InputValue::Field(match width {
            8 => neg_as_to_field::<i8>(input),
            16 => neg_as_to_field::<i16>(input),
            32 => neg_as_to_field::<i32>(input),
            64 => neg_as_to_field::<i64>(input),
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
            _ => {
                panic!("Shouldn't be reachable")
            }
        })
    }

    /// Shift signed value
    fn shift_signed_int(&mut self, input: &i128, width: u32) -> InputValue {
        let exponent = self.prng.gen_range(1..=(width * 2 - 1));
        InputValue::Field(match width {
            8 => shift::<i8>(input, exponent),
            16 => shift::<i16>(input, exponent),
            32 => shift::<i32>(input, exponent),
            64 => shift::<i64>(input, exponent),
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
        InputValue::Field(width_dictionary.choose(self.prng).unwrap().clone())
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
        let chosen_operation = BASIC_BINARY_OPERATION_MUTATION_CONFIGURATION.select(self.prng);
        InputValue::Field(match width {
            8 => add_sub_xor_and_or_signed::<i8>(input, &dictionary_value, chosen_operation),
            16 => add_sub_xor_and_or_signed::<i16>(input, &dictionary_value, chosen_operation),
            32 => add_sub_xor_and_or_signed::<i32>(input, &dictionary_value, chosen_operation),
            64 => add_sub_xor_and_or_signed::<i64>(input, &dictionary_value, chosen_operation),
            _ => {
                panic!("Shouldn't be reachable")
            }
        })
    }

    /// Perform a mutation on a signed int
    fn mutate_signed(&mut self, input: &FieldElement, width: u32) -> InputValue {
        let initial_i128 = field_to_i128(&input, width);
        match BASIC_INT_TOP_LEVEL_MUTATION_CONFIGURATION.select(self.prng) {
            IntTopLevelMutation::FixedSubstitution => {
                self.substitute_signed_int_with_fixed_value(width)
            }
            IntTopLevelMutation::DictionarySubstitution => {
                self.substitute_with_dictionary_value(width)
            }
            IntTopLevelMutation::Negation => self.negate_signed_int(&initial_i128, width),
            IntTopLevelMutation::Shift => self.shift_signed_int(&initial_i128, width),
            IntTopLevelMutation::SmallValueUpdate => {
                self.sub_add_small_value_signed(&initial_i128, width)
            }
            IntTopLevelMutation::DictionaryValueUpdate => {
                self.perform_signed_binary_operation_with_dictionary(input, width)
            }
        }
    }

    // Get one of the fixed values in place of the original value
    fn substitute_unsigned_int_with_fixed_value(&mut self, width: u32) -> InputValue {
        let result_subst = InputValue::Field(FieldElement::from(
            FIXED_UNSIGNED_VALUES[self.prng.gen_range(0..(width * 4) as usize)]
                & ((1i128 << width) - 1),
        ));
        return result_subst;
    }

    /// Negate an unsigned value
    fn negate_unsigned_int(&mut self, input: &i128, width: u32) -> InputValue {
        InputValue::Field(match width {
            8 => wrapping_neg_as_to_field::<u8>(input),
            16 => wrapping_neg_as_to_field::<u16>(input),
            32 => wrapping_neg_as_to_field::<u32>(input),
            64 => wrapping_neg_as_to_field::<u64>(input),
            _ => {
                panic!("Shouldn't be reachable")
            }
        })
    }

    /// Shift signed value
    fn shift_unsigned_int(&mut self, input: &i128, width: u32) -> InputValue {
        let exponent = self.prng.gen_range(1..=(width * 2 - 1));
        InputValue::Field(match width {
            8 => shift::<u8>(input, exponent),
            16 => shift::<u16>(input, exponent),
            32 => shift::<u32>(input, exponent),
            64 => shift::<u64>(input, exponent),
            _ => {
                panic!("Shouldn't be reachable")
            }
        })
    }

    /// Add or subtract a small signed value to input
    fn sub_add_small_value_unsigned(&mut self, input: &i128, width: u32) -> InputValue {
        let update = self.prng.gen_range(u8::MIN..=u8::MAX);

        // Probability of addition and subtraction is equal
        if self.prng.gen_range(0..2).is_zero() {
            return InputValue::Field(match width {
                8 => wrapping_add_small_unsigned::<u8>(input, update),
                16 => wrapping_add_small_unsigned::<u16>(input, update),
                32 => wrapping_add_small_unsigned::<u32>(input, update),
                64 => wrapping_add_small_unsigned::<u64>(input, update),
                _ => {
                    panic!("Shouldn't be reachable")
                }
            });
        } else {
            return InputValue::Field(match width {
                8 => wrapping_sub_small_unsigned::<u8>(input, update),
                16 => wrapping_sub_small_unsigned::<u16>(input, update),
                32 => wrapping_sub_small_unsigned::<u32>(input, update),
                64 => wrapping_sub_small_unsigned::<u64>(input, update),
                _ => {
                    panic!("Shouldn't be reachable")
                }
            });
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
        let chosen_operation = BASIC_BINARY_OPERATION_MUTATION_CONFIGURATION.select(self.prng);
        InputValue::Field(match width {
            8 => add_sub_xor_and_or_unsigned::<u8>(input, &dictionary_value, chosen_operation),
            16 => add_sub_xor_and_or_unsigned::<u16>(input, &dictionary_value, chosen_operation),
            32 => add_sub_xor_and_or_unsigned::<u32>(input, &dictionary_value, chosen_operation),
            64 => add_sub_xor_and_or_unsigned::<u64>(input, &dictionary_value, chosen_operation),
            _ => {
                panic!("Shouldn't be reachable")
            }
        })
    }
    /// Perform a mutation on an unsigned int
    pub fn mutate_unsigned(&mut self, input: &FieldElement, width: u32) -> InputValue {
        let initial_i128 = field_to_i128(&input, width + 1);
        match BASIC_INT_TOP_LEVEL_MUTATION_CONFIGURATION.select(self.prng) {
            IntTopLevelMutation::FixedSubstitution => {
                self.substitute_unsigned_int_with_fixed_value(width)
            }
            IntTopLevelMutation::DictionarySubstitution => {
                self.substitute_with_dictionary_value(width)
            }
            IntTopLevelMutation::Negation => self.negate_unsigned_int(&initial_i128, width),
            IntTopLevelMutation::Shift => self.shift_unsigned_int(&initial_i128, width),
            IntTopLevelMutation::SmallValueUpdate => {
                self.sub_add_small_value_unsigned(&initial_i128, width)
            }
            IntTopLevelMutation::DictionaryValueUpdate => {
                self.perform_unsigned_binary_operation_with_dictionary(input, width)
            }
        }
    }

    /// Mutate an input value depending on the sign and width
    pub fn mutate(&mut self, input: &InputValue, sign: &Sign, width: u32) -> InputValue {
        let initial_field_value = match input {
            InputValue::Field(inner_field) => inner_field,
            _ => panic!("Shouldn't be used with other input value types"),
        }
        .clone();
        assert!(width == 1 || width == 8 || width == 16 || width == 32 || width == 64);

        // If it's just one bit just get a random value
        if width == 1 {
            assert!(*sign == Sign::Unsigned);
            return InputValue::Field(FieldElement::from(self.prng.gen_range(0..1u32)));
        } else {
            match sign {
                Sign::Signed => self.mutate_signed(&initial_field_value, width),
                Sign::Unsigned => self.mutate_unsigned(&initial_field_value, width),
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
