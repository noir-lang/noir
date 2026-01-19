//! This file contains mechanisms for deterministically mutating a given field value
//! Types of mutations applied:
//! 1. Substitutions
//!     1. With zero
//!     2. With one
//!     3. With minus one
//!     4. With a value from the dictionary (created from analyzing the code of the program and testcases in the corpus)
//!     5. With a power of 2
//!     6. With a power of 2 minus 1
//! 2. Inversions
//!     1. Negation
//!     2. Multiplicative inverse
//! 3. Update with a value that is a power of 2
//!     1. Addition
//!     2. Subtraction
//!     3. Multiplication
//!     4. Division
//! 4. Update with a small (1-255) value
//!     1. Addition
//!     2. Subtraction
//!     3. Multiplication
//! 5. Update with a dictionary value
//!     1. Addition
//!     2. Subtraction
//!     3. Multiplication
//!
//! There are configurations for determining probability of each top-level and low-level mutation
//! Currently, the configurations are constant and "new" methods aren't used, but the architecture is prepared for easier introduction of MOpt (Mutation Optimization) technique in the future

use std::sync::OnceLock;

use super::configurations::{
    BASIC_FIELD_ELEMENT_DICTIONARY_UPDATE_CONFIGURATION,
    BASIC_FIELD_ELEMENT_POW_2_UPDATE_CONFIGURATION,
    BASIC_FIELD_ELEMENT_SMALL_VALUE_UPDATE_CONFIGURATION, BASIC_FIELD_INVERSION_CONFIGURATION,
    BASIC_FIELD_SUBSTITUTION_CONFIGURATION, BASIC_TOPLEVEL_FIELD_ELEMENT_MUTATION_CONFIGURATION,
    FieldElementDictionaryUpdateOptions, FieldElementInversionMutationOptions,
    FieldElementPow2UpdateOptions, FieldElementSmallValueUpdateOptions,
    FieldElementSubstitutionMutationOptions, TopLevelFieldElementMutationOptions,
};
use acvm::{AcirField, FieldElement};
use noirc_abi::input_parser::InputValue;
use rand::{Rng, seq::IndexedRandom};
use rand_xorshift::XorShiftRng;

const SMALL_VALUE_MAX: u64 = 0xff;
const SMALL_VALUE_MIN: u64 = 1;

fn powers_of_two() -> &'static Vec<FieldElement> {
    static INSTANCE: OnceLock<Vec<FieldElement>> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        (1..=MAX_POW_2)
            .map(|i| FieldElement::from(2i128).pow(&FieldElement::from(i)))
            .collect::<Vec<FieldElement>>()
    })
}

fn powers_of_two_minus_one() -> &'static Vec<FieldElement> {
    static INSTANCE: OnceLock<Vec<FieldElement>> = OnceLock::new();
    INSTANCE.get_or_init(|| powers_of_two().iter().map(|x| *x - FieldElement::one()).collect())
}

fn inverse_powers_of_two() -> &'static Vec<FieldElement> {
    static INSTANCE: OnceLock<Vec<FieldElement>> = OnceLock::new();
    INSTANCE
        .get_or_init(|| powers_of_two().iter().map(|p| p.inverse()).collect::<Vec<FieldElement>>())
}

// We are using bn254 scalar field so 254 is enough
const MAX_POW_2: usize = 254;

struct FieldMutator<'a> {
    dictionary: &'a Vec<FieldElement>,
    prng: &'a mut XorShiftRng,
}

impl<'a> FieldMutator<'a> {
    pub fn new(dictionary: &'a Vec<FieldElement>, prng: &'a mut XorShiftRng) -> Self {
        // Initialize powers of two if we haven't done that yet
        assert!(!dictionary.is_empty());
        Self { dictionary, prng }
    }

    #[allow(static_mut_refs)]
    fn apply_substitution(&mut self) -> FieldElement {
        match BASIC_FIELD_SUBSTITUTION_CONFIGURATION.select(self.prng) {
            FieldElementSubstitutionMutationOptions::Zero => FieldElement::from(0u32),
            FieldElementSubstitutionMutationOptions::One => FieldElement::from(1u32),
            FieldElementSubstitutionMutationOptions::MinusOne => -FieldElement::from(1u32),
            FieldElementSubstitutionMutationOptions::Dictionary => {
                *self.dictionary.choose(self.prng).unwrap()
            }
            FieldElementSubstitutionMutationOptions::PowerOfTwo => {
                *powers_of_two().choose(self.prng).unwrap()
            }
            FieldElementSubstitutionMutationOptions::PowerOfTwoMinusOne => {
                *powers_of_two_minus_one().choose(self.prng).unwrap()
            }
        }
    }

    fn apply_inversion(&mut self, element: FieldElement) -> FieldElement {
        match BASIC_FIELD_INVERSION_CONFIGURATION.select(self.prng) {
            FieldElementInversionMutationOptions::Additive => -element,
            FieldElementInversionMutationOptions::Multiplicative => element.inverse(),
        }
    }

    #[allow(static_mut_refs)]
    fn apply_pow_2_update(&mut self, element: FieldElement) -> FieldElement {
        let chosen_power_of_two = powers_of_two().choose(self.prng).unwrap();
        let chosen_inverse_power_of_two = inverse_powers_of_two().choose(self.prng).unwrap();
        match BASIC_FIELD_ELEMENT_POW_2_UPDATE_CONFIGURATION.select(self.prng) {
            FieldElementPow2UpdateOptions::Addition => element + *chosen_power_of_two,
            FieldElementPow2UpdateOptions::Subtraction => element - *chosen_power_of_two,
            FieldElementPow2UpdateOptions::Multiplication => element * *chosen_power_of_two,
            FieldElementPow2UpdateOptions::Division => element * *chosen_inverse_power_of_two,
        }
    }

    fn apply_small_value_update(&mut self, element: FieldElement) -> FieldElement {
        let small_value =
            FieldElement::from(self.prng.random_range(SMALL_VALUE_MIN..=SMALL_VALUE_MAX));
        match BASIC_FIELD_ELEMENT_SMALL_VALUE_UPDATE_CONFIGURATION.select(self.prng) {
            FieldElementSmallValueUpdateOptions::Addition => element + small_value,
            FieldElementSmallValueUpdateOptions::Subtraction => element - small_value,
            FieldElementSmallValueUpdateOptions::Multiplication => element * small_value,
        }
    }

    fn apply_dictionary_update(&mut self, element: FieldElement) -> FieldElement {
        let dictionary_value = self.dictionary.choose(self.prng).unwrap();
        match BASIC_FIELD_ELEMENT_DICTIONARY_UPDATE_CONFIGURATION.select(self.prng) {
            FieldElementDictionaryUpdateOptions::Addition => element + *dictionary_value,
            FieldElementDictionaryUpdateOptions::Subtraction => element - *dictionary_value,
            FieldElementDictionaryUpdateOptions::Multiplication => element * *dictionary_value,
        }
    }

    pub fn mutate(&mut self, input: &InputValue) -> InputValue {
        let initial_field_value = match input {
            InputValue::Field(inner_field) => *inner_field,
            _ => panic!("Shouldn't be used with other input value types"),
        };
        InputValue::Field(
            match BASIC_TOPLEVEL_FIELD_ELEMENT_MUTATION_CONFIGURATION.select(self.prng) {
                TopLevelFieldElementMutationOptions::Substitution => self.apply_substitution(),
                TopLevelFieldElementMutationOptions::Inversion => {
                    self.apply_inversion(initial_field_value)
                }
                TopLevelFieldElementMutationOptions::Pow2Update => {
                    self.apply_pow_2_update(initial_field_value)
                }
                TopLevelFieldElementMutationOptions::SmallValueUpdate => {
                    self.apply_small_value_update(initial_field_value)
                }
                TopLevelFieldElementMutationOptions::DictionaryUpdate => {
                    self.apply_dictionary_update(initial_field_value)
                }
            },
        )
    }
}
/// Mutates a field element input value using various mutation strategies.
///
/// This function takes an existing field element input value and applies one of several
/// possible mutations to generate a new value. The mutations are selected randomly and can include:
/// - Substitution with a random value
/// - Inversion
/// - Addition/subtraction/multiplication with powers of 2
/// - Addition/subtraction/multiplication with small values
/// - Addition/subtraction/multiplication with dictionary values
///
/// # Arguments
///
/// * `previous_input` - The input value to mutate, must be a Field variant
/// * `dictionary` - A vector of interesting field element values to use in mutations
/// * `prng` - Random number generator for selecting mutations
pub(super) fn mutate_field_input_value(
    previous_input: &InputValue,
    dictionary: &Vec<FieldElement>,
    prng: &mut XorShiftRng,
) -> InputValue {
    let mut field_mutator = FieldMutator::new(dictionary, prng);
    field_mutator.mutate(previous_input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;
    use rand::SeedableRng;

    fn create_dictionary() -> Vec<FieldElement> {
        vec![FieldElement::from(42u32)]
    }

    #[test]
    #[allow(static_mut_refs)]
    fn test_apply_substitution() {
        let mut prng = XorShiftRng::seed_from_u64(0);
        let dictionary = create_dictionary();
        let mut mutator = FieldMutator::new(&dictionary, &mut prng);

        let result = mutator.apply_substitution();
        // Test that result is one of the possible substitution values
        assert!(
            result == FieldElement::from(0u32)
                || result == FieldElement::from(1u32)
                || result == -FieldElement::from(1u32)
                || result == FieldElement::from(42u32)
                || powers_of_two().contains(&result)
                || powers_of_two_minus_one().contains(&result)
        );
    }

    #[test]
    fn test_apply_inversion() {
        let mut prng = XorShiftRng::seed_from_u64(0);
        let dictionary = create_dictionary();
        let mut mutator = FieldMutator::new(&dictionary, &mut prng);
        let element = FieldElement::from(10u32);

        for _ in 0..10 {
            let result = mutator.apply_inversion(element);
            // Test that result is either additive or multiplicative inverse
            assert!(result == -element || result == element.inverse());
        }
    }

    #[test]
    #[allow(static_mut_refs)]
    fn test_apply_pow_2_update() {
        let mut prng = XorShiftRng::seed_from_u64(0);
        let dictionary = create_dictionary();
        let mut mutator = FieldMutator::new(&dictionary, &mut prng);
        let element = FieldElement::from(10u32);

        for _ in 0..10 {
            let result = mutator.apply_pow_2_update(element);
            // Verify result is different from input
            assert_ne!(result, element);
            // Result should be element combined with a power of 2 via +,-,*,/
            assert!(powers_of_two().iter().any(|p| {
                result == element + *p
                    || result == element - *p
                    || result == element * *p
                    || result == element * p.inverse()
            }));
        }
    }

    #[test]
    fn test_apply_small_value_update() {
        let mut prng = XorShiftRng::seed_from_u64(0);
        let dictionary = create_dictionary();
        let mut mutator = FieldMutator::new(&dictionary, &mut prng);
        let element = FieldElement::from(10u32);

        for _ in 0..10 {
            let result = mutator.apply_small_value_update(element);
            // Verify result is different from input
            assert_ne!(result, element);
            let diff = result - element;
            // Check both addition/subtraction and multiplication cases
            let ratio =
                if element != FieldElement::zero() { result * element.inverse() } else { result };
            assert!(
                BigUint::from_bytes_be(&(diff * diff).to_be_bytes())
                    <= BigUint::from(SMALL_VALUE_MAX * SMALL_VALUE_MAX)
                    || BigUint::from_bytes_be(&(ratio).to_be_bytes())
                        <= BigUint::from(SMALL_VALUE_MAX)
            );
        }
    }

    #[test]
    fn test_apply_dictionary_update() {
        let mut prng = XorShiftRng::seed_from_u64(0);
        let dictionary = create_dictionary();
        let mut mutator = FieldMutator::new(&dictionary, &mut prng);
        let element = FieldElement::from(10u32);

        for _ in 0..10 {
            let result = mutator.apply_dictionary_update(element);
            // Verify result is different from input
            assert_ne!(result, element);
            // Result should be element combined with dictionary value
            let dict_val = dictionary[0];
            assert!(
                result == element + dict_val
                    || result == element - dict_val
                    || result == element * dict_val
            );
        }
    }
    #[test]
    fn test_mutate_field_input_value() {
        let mut prng = XorShiftRng::seed_from_u64(0);
        let dictionary = vec![FieldElement::one()];

        let mut input = InputValue::Field(FieldElement::one());
        let mut changes = 0;

        for _ in 0..100 {
            let result = mutate_field_input_value(&input, &dictionary, &mut prng);

            // Verify result is a field element
            match result {
                InputValue::Field(_) => (),
                _ => panic!("Expected Field variant"),
            }

            if input != result {
                changes += 1;
            }

            input = result;
        }

        // Verify at least 90% of mutations produced different values
        assert!(changes >= 90, "Only {changes} out of 100 mutations produced different values");
    }
}
