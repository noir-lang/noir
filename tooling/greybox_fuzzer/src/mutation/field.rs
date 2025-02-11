use super::configurations::{
    FieldElementDictionaryUpdate, FieldElementInversionMutation, FieldElementPow2Update,
    FieldElementSmallValueUpdate, FieldElementSubstitutionMutation, TopLevelFieldElementMutation,
    BASIC_FIELD_ELEMENT_DICTIONARY_UPDATE_CONFIGURATION,
    BASIC_FIELD_ELEMENT_POW_2_UPDATE_CONFIGURATION,
    BASIC_FIELD_ELEMENT_SMALL_VALUE_UPDATE_CONFIGURATION, BASIC_FIELD_INVERSION_CONFIGURATION,
    BASIC_FIELD_SUBSTITUTION_CONFIGURATION, BASIC_TOPLEVEL_FIELD_ELEMENT_MUTATION_CONFIGURATION,
};
use acvm::{AcirField, FieldElement};
use noirc_abi::input_parser::InputValue;
use rand::{seq::SliceRandom, Rng};
use rand_xorshift::XorShiftRng;

/// This file contains mechanisms for deterministically mutating a given field value
/// Types of mutations applied:
/// 1. Substitutions
///     a. With zero
///     b. With one
///     c. With minus one
///     d. With a value from the dictionary (created from analyzing the code of the program and testcases in the corpus)
///     e. With a power of 2
///     f. With a power of 2 minus 1
/// 2. Inversions
///     a. Negation
///     b. Multiplicative inverse
/// 3. Update with a value that is a power of 2
///     a. Addition
///     b. Subtraction
///     c. Multiplication
///     d. Division
/// 4. Update with a small (1-255) value
///     a. Addition
///     b. Subtraction
///     c. Multiplication
/// 5. Update with a dictionary value
///     a. Addition
///     b. Subtraction
///     c. Multiplication
///
/// There are configurations for determining probability of each top-level and low-level mutation
/// Currently, the configurations are constant and "new" methods aren't used, but the architecture is prepared for easier introduction of MOpt (Mutation Optimization) technique in the future
///
const SMALL_VALUE_MAX: u64 = 0xff;
const SMALL_VALUE_MIN: u64 = 1;

static mut POWERS_OF_TWO_INITIALIZED: bool = false;
static mut POWERS_OF_TWO: Vec<FieldElement> = Vec::new();
static mut INVERSE_POWERS_OF_TWO: Vec<FieldElement> = Vec::new();
static mut POWERS_OF_TWO_MINUS_ONE: Vec<FieldElement> = Vec::new();

// We are using bn254 scalar field so 254 is enough
const MAX_POW_2: usize = 254;

struct FieldMutator<'a> {
    dictionary: &'a Vec<FieldElement>,
    prng: &'a mut XorShiftRng,
}

impl<'a> FieldMutator<'a> {
    pub fn new(dictionary: &'a Vec<FieldElement>, prng: &'a mut XorShiftRng) -> Self {
        // Initialize powers of two if we haven't done that yet
        unsafe {
            if !POWERS_OF_TWO_INITIALIZED {
                let powers_of_two = (1..=MAX_POW_2)
                    .map(|i| FieldElement::from(2i128).pow(&FieldElement::from(i)))
                    .collect::<Vec<FieldElement>>();
                INVERSE_POWERS_OF_TWO =
                    powers_of_two.iter().map(|p| p.inverse()).collect::<Vec<FieldElement>>();

                POWERS_OF_TWO_MINUS_ONE =
                    powers_of_two.iter().map(|x| *x - FieldElement::from(1i128)).collect();

                POWERS_OF_TWO = powers_of_two;
                POWERS_OF_TWO_INITIALIZED = true;
            }
        };

        assert!(!dictionary.is_empty());
        Self { dictionary, prng }
    }

    fn apply_substitution(&mut self) -> FieldElement {
        match BASIC_FIELD_SUBSTITUTION_CONFIGURATION.select(self.prng) {
            FieldElementSubstitutionMutation::Zero => FieldElement::from(0u32),
            FieldElementSubstitutionMutation::One => FieldElement::from(1u32),
            FieldElementSubstitutionMutation::MinusOne => -FieldElement::from(1u32),
            FieldElementSubstitutionMutation::Dictionary => {
                *self.dictionary.choose(self.prng).unwrap()
            }
            FieldElementSubstitutionMutation::PowerOfTwo => unsafe {
                *POWERS_OF_TWO.choose(self.prng).unwrap()
            },
            FieldElementSubstitutionMutation::PowerOfTwoMinusOne => unsafe {
                *POWERS_OF_TWO_MINUS_ONE.choose(self.prng).unwrap()
            },
        }
    }

    fn apply_inversion(&mut self, element: FieldElement) -> FieldElement {
        match BASIC_FIELD_INVERSION_CONFIGURATION.select(self.prng) {
            FieldElementInversionMutation::Additive => -element,
            FieldElementInversionMutation::Multiplicative => element.inverse(),
        }
    }

    fn apply_pow_2_update(&mut self, element: FieldElement) -> FieldElement {
        let chosen_power_of_two = unsafe { POWERS_OF_TWO.choose(self.prng).unwrap() };
        let chosen_inverse_power_of_two =
            unsafe { INVERSE_POWERS_OF_TWO.choose(self.prng).unwrap() };
        match BASIC_FIELD_ELEMENT_POW_2_UPDATE_CONFIGURATION.select(self.prng) {
            FieldElementPow2Update::Addition => element + *chosen_power_of_two,
            FieldElementPow2Update::Subtraction => element - *chosen_power_of_two,
            FieldElementPow2Update::Multiplication => element * *chosen_power_of_two,
            FieldElementPow2Update::Division => element * *chosen_inverse_power_of_two,
        }
    }

    fn apply_small_value_update(&mut self, element: FieldElement) -> FieldElement {
        let small_value =
            FieldElement::from(self.prng.gen_range(SMALL_VALUE_MIN..=SMALL_VALUE_MAX));
        match BASIC_FIELD_ELEMENT_SMALL_VALUE_UPDATE_CONFIGURATION.select(self.prng) {
            FieldElementSmallValueUpdate::Addition => element + small_value,
            FieldElementSmallValueUpdate::Subtraction => element - small_value,
            FieldElementSmallValueUpdate::Multiplication => element * small_value,
        }
    }

    fn apply_dictionary_update(&mut self, element: FieldElement) -> FieldElement {
        let dictionary_value = self.dictionary.choose(self.prng).unwrap();
        match BASIC_FIELD_ELEMENT_DICTIONARY_UPDATE_CONFIGURATION.select(self.prng) {
            FieldElementDictionaryUpdate::Addition => element + *dictionary_value,
            FieldElementDictionaryUpdate::Subtraction => element - *dictionary_value,
            FieldElementDictionaryUpdate::Multiplication => element * *dictionary_value,
        }
    }

    pub fn mutate(&mut self, input: &InputValue) -> InputValue {
        let initial_field_value = match input {
            InputValue::Field(inner_field) => *inner_field,
            _ => panic!("Shouldn't be used with other input value types"),
        };
        InputValue::Field(
            match BASIC_TOPLEVEL_FIELD_ELEMENT_MUTATION_CONFIGURATION.select(self.prng) {
                TopLevelFieldElementMutation::Substitution => self.apply_substitution(),
                TopLevelFieldElementMutation::Inversion => {
                    self.apply_inversion(initial_field_value)
                }
                TopLevelFieldElementMutation::Pow2Update => {
                    self.apply_pow_2_update(initial_field_value)
                }
                TopLevelFieldElementMutation::SmallValueUpdate => {
                    self.apply_small_value_update(initial_field_value)
                }
                TopLevelFieldElementMutation::DictionaryUpdate => {
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
pub fn mutate_field_input_value(
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
    use rand::SeedableRng;

    fn create_dictionary() -> Vec<FieldElement> {
        vec![FieldElement::from(42u32)]
    }

    #[test]
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
                || unsafe { POWERS_OF_TWO.contains(&result) }
                || unsafe { POWERS_OF_TWO_MINUS_ONE.contains(&result) }
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
            assert!(unsafe {
                POWERS_OF_TWO.iter().any(|p| {
                    result == element + *p
                        || result == element - *p
                        || result == element * *p
                        || result == element * p.inverse()
                })
            });
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
