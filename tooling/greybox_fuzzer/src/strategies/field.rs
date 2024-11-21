use acvm::{AcirField, FieldElement};

use noirc_abi::input_parser::InputValue;

use rand::{seq::SliceRandom, Rng};
use rand_xorshift::XorShiftRng;

const SUBSTITUTION_BY_ZERO_WEIGHT: usize = 20;
const SUBSTITUTION_BY_ONE_WEIGHT: usize = 20;
const SUBSTITUTION_BY_MINUS_ONE_WEIGHT: usize = 20;
const SUBSTITUTION_FROM_DICTIONARY_WEIGHT: usize = 20;
const MAX_POW_2: usize = 254;
static mut POWERS_OF_TWO_INITIALIZED: bool = false;
static mut POWERS_OF_TWO: Vec<FieldElement> = Vec::new();
static mut INVERSE_POWERS_OF_TWO: Vec<FieldElement> = Vec::new();
static mut POWERS_OF_TWO_MINUS_ONE: Vec<FieldElement> = Vec::new();

const SUBSTITUTION_WEIGHT: usize = SUBSTITUTION_BY_ZERO_WEIGHT
    + SUBSTITUTION_BY_MINUS_ONE_WEIGHT
    + SUBSTITUTION_BY_ONE_WEIGHT
    + SUBSTITUTION_FROM_DICTIONARY_WEIGHT
    + 2 * MAX_POW_2;

const ADDITIVE_INVERSION_WEIGHT: usize = 50;
// Heavy
const MULTIPLICATIVE_INVERSION_WEIGHT: usize = 5;

const INVERSION_WEIGHT: usize = ADDITIVE_INVERSION_WEIGHT + MULTIPLICATIVE_INVERSION_WEIGHT;

const ADDITION_OF_POW_2_WEIGHT: usize = MAX_POW_2;
const SUBTRACTION_OF_POW_2_WEIGHT: usize = MAX_POW_2;
const MULTIPLICATION_OF_POW_2_WEIGHT: usize = MAX_POW_2;
const DIVISION_OF_POW_2_WEIGHT: usize = MAX_POW_2;

const POW_2_UPDATE_WEIGHT: usize = ADDITION_OF_POW_2_WEIGHT
    + SUBTRACTION_OF_POW_2_WEIGHT
    + MULTIPLICATION_OF_POW_2_WEIGHT
    + DIVISION_OF_POW_2_WEIGHT;

const SMALL_VALUE_MAX: u64 = 0xff;
const SMALL_VALUE_MIN: u64 = 1;
const ADDITION_OF_A_SMALL_VALUE_WEIGHT: usize = 0x100;
const SUBTRACTION_OF_A_SMALL_VALUE_WEIGHT: usize = 0x100;
const MULTIPLICATION_BY_A_SMALL_VALUE_WEIGHT: usize = 0x80;

const SMALL_VALUE_UPDATE_WEIGHT: usize = ADDITION_OF_A_SMALL_VALUE_WEIGHT
    + SUBTRACTION_OF_A_SMALL_VALUE_WEIGHT
    + MULTIPLICATION_BY_A_SMALL_VALUE_WEIGHT;

const ADDITION_OF_DICTIONARY_VALUE_WEIGHT: usize = 0x40;
const SUBTRACTION_OF_DICTIONARY_VALUE_WEIGHT: usize = 0x40;
const MULTIPLICATION_BY_DICTIONARY_VALUE_WEIGHT: usize = 0x40;

const DICTIONARY_ARITHMETIC_UPDATE_WEIGHT: usize = ADDITION_OF_DICTIONARY_VALUE_WEIGHT
    + SUBTRACTION_OF_DICTIONARY_VALUE_WEIGHT
    + MULTIPLICATION_BY_DICTIONARY_VALUE_WEIGHT;

const TOTAL_WEIGHT: usize = SUBSTITUTION_WEIGHT
    + INVERSION_WEIGHT
    + POW_2_UPDATE_WEIGHT
    + SMALL_VALUE_UPDATE_WEIGHT
    + DICTIONARY_ARITHMETIC_UPDATE_WEIGHT;

fn initialize_powers_of_two() {
    unsafe {
        if !POWERS_OF_TWO_INITIALIZED {
            POWERS_OF_TWO_INITIALIZED = true;
            POWERS_OF_TWO = (1..=MAX_POW_2)
                .map(|i| FieldElement::from(2i128).pow(&FieldElement::from(i)))
                .collect::<Vec<FieldElement>>();
            INVERSE_POWERS_OF_TWO =
                POWERS_OF_TWO.iter().map(|p| p.inverse()).collect::<Vec<FieldElement>>();
            POWERS_OF_TWO_MINUS_ONE =
                POWERS_OF_TWO.iter().map(|x| *x - FieldElement::from(1i128)).collect();
        }
    }
}
pub fn mutate_field_input_value(
    previous_input: &InputValue,
    dictionary: &Vec<FieldElement>,
    prng: &mut XorShiftRng,
) -> InputValue {
    initialize_powers_of_two();
    let initial_field_value = match previous_input {
        InputValue::Field(inner_field) => inner_field,
        _ => panic!("Shouldn't be used with other input value types"),
    }
    .clone();
    assert!(!dictionary.is_empty());
    // Types of mutations:
    // 1. Substitution (0, 1, -1, powers of 2, powers of 2 minus one)
    // 2. Negation/inversion
    // 3. Multiplication by a power of 2 (division by a power of 2)
    // 4. Addition/subtraction of a small value
    // 5. Additions/subtraction of a power of two
    // 6. Addition/Substitution of a Value from the dictionary
    let mut selector = prng.gen_range(0..TOTAL_WEIGHT);

    // Substitutions
    if selector < SUBSTITUTION_WEIGHT {
        // By Zero
        if selector < SUBSTITUTION_BY_ZERO_WEIGHT {
            return InputValue::Field(FieldElement::from(0u32));
        }
        selector -= SUBSTITUTION_BY_ONE_WEIGHT;
        // By one
        if selector < SUBSTITUTION_BY_ONE_WEIGHT {
            return InputValue::Field(FieldElement::from(1u32));
        }
        selector -= SUBSTITUTION_BY_MINUS_ONE_WEIGHT;
        // By minus one
        if selector < SUBSTITUTION_BY_ONE_WEIGHT {
            return InputValue::Field(-FieldElement::from(1u32));
        }
        selector -= SUBSTITUTION_BY_ONE_WEIGHT;
        // By a random value from dictionary
        if selector < SUBSTITUTION_FROM_DICTIONARY_WEIGHT {
            return InputValue::Field(*dictionary.choose(prng).unwrap());
        }
        selector -= SUBSTITUTION_FROM_DICTIONARY_WEIGHT;
        unsafe {
            // By a power of two
            if selector < MAX_POW_2 {
                return InputValue::Field(POWERS_OF_TWO[selector]);
            }
            selector -= MAX_POW_2;
            assert!(selector < MAX_POW_2);
            // By a power of two minus one
            return InputValue::Field(POWERS_OF_TWO_MINUS_ONE[selector]);
        }
    }
    selector -= SUBSTITUTION_WEIGHT;
    // Inverses
    if selector < INVERSION_WEIGHT {
        // Negation
        if selector < ADDITIVE_INVERSION_WEIGHT {
            return InputValue::Field(-initial_field_value);
        }
        selector -= ADDITIVE_INVERSION_WEIGHT;
        assert!(selector < MULTIPLICATIVE_INVERSION_WEIGHT);
        // Multiplicative inverse
        return InputValue::Field(initial_field_value.inverse());
    }
    selector -= INVERSION_WEIGHT;
    unsafe {
        // Additions, subtractions, etc of powers of two
        if selector < POW_2_UPDATE_WEIGHT {
            // An addition of a power of two
            if selector < ADDITION_OF_POW_2_WEIGHT {
                return InputValue::Field(
                    initial_field_value + POWERS_OF_TWO[prng.gen_range(0..MAX_POW_2)],
                );
            }
            selector -= ADDITION_OF_POW_2_WEIGHT;
            // Subtraction of a power of two
            if selector < SUBTRACTION_OF_POW_2_WEIGHT {
                return InputValue::Field(
                    initial_field_value - POWERS_OF_TWO[prng.gen_range(0..MAX_POW_2)],
                );
            }
            // Multiplication by a power of two (bit shift left)
            selector -= SUBTRACTION_OF_POW_2_WEIGHT;
            if selector < MULTIPLICATION_OF_POW_2_WEIGHT {
                return InputValue::Field(
                    initial_field_value * POWERS_OF_TWO[prng.gen_range(0..MAX_POW_2)],
                );
            }
            // Division by a power of two
            selector -= MULTIPLICATION_OF_POW_2_WEIGHT;
            assert!(selector < DIVISION_OF_POW_2_WEIGHT);
            return InputValue::Field(
                initial_field_value * INVERSE_POWERS_OF_TWO[prng.gen_range(0..MAX_POW_2)],
            );
        }
    }
    selector -= POW_2_UPDATE_WEIGHT;

    // Updates with addition, subtraction or multiplication by small values
    if selector < SMALL_VALUE_UPDATE_WEIGHT {
        if selector < ADDITION_OF_A_SMALL_VALUE_WEIGHT {
            return InputValue::Field(
                initial_field_value
                    + FieldElement::from(prng.gen_range(SMALL_VALUE_MIN..=SMALL_VALUE_MAX)),
            );
        }
        selector -= ADDITION_OF_A_SMALL_VALUE_WEIGHT;
        if selector < SUBTRACTION_OF_A_SMALL_VALUE_WEIGHT {
            return InputValue::Field(
                initial_field_value
                    - FieldElement::from(prng.gen_range(SMALL_VALUE_MIN..=SMALL_VALUE_MAX)),
            );
        }
        selector -= SUBTRACTION_OF_A_SMALL_VALUE_WEIGHT;
        assert!(selector < MULTIPLICATION_BY_A_SMALL_VALUE_WEIGHT);
        return InputValue::Field(
            initial_field_value
                * FieldElement::from(prng.gen_range(SMALL_VALUE_MIN..=SMALL_VALUE_MAX)),
        );
    }
    selector -= SMALL_VALUE_UPDATE_WEIGHT;

    assert!(selector < DICTIONARY_ARITHMETIC_UPDATE_WEIGHT);
    if selector < ADDITION_OF_DICTIONARY_VALUE_WEIGHT {
        return InputValue::Field(initial_field_value + *dictionary.choose(prng).unwrap());
    }
    selector -= ADDITION_OF_DICTIONARY_VALUE_WEIGHT;
    if selector < SUBTRACTION_OF_DICTIONARY_VALUE_WEIGHT {
        return InputValue::Field(initial_field_value - *dictionary.choose(prng).unwrap());
    }
    selector -= SUBTRACTION_OF_A_SMALL_VALUE_WEIGHT;
    assert!(selector < MULTIPLICATION_BY_DICTIONARY_VALUE_WEIGHT);
    return InputValue::Field(initial_field_value * *dictionary.choose(prng).unwrap());
}
