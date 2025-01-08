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

/// Initialize a static vector of powers of two for quick access during mutations
fn initialize_powers_of_two() {
    unsafe {
        if !POWERS_OF_TWO_INITIALIZED {
            POWERS_OF_TWO_INITIALIZED = true;
            let powers_of_two = (1..=MAX_POW_2)
                .map(|i| FieldElement::from(2i128).pow(&FieldElement::from(i)))
                .collect::<Vec<FieldElement>>();
            INVERSE_POWERS_OF_TWO =
                powers_of_two.iter().map(|p| p.inverse()).collect::<Vec<FieldElement>>();

            POWERS_OF_TWO_MINUS_ONE =
                powers_of_two.iter().map(|x| *x - FieldElement::from(1i128)).collect();

            POWERS_OF_TWO = powers_of_two;
        }
    }
}
enum SubstitutionMutation {
    Zero,
    One,
    MinusOne,
    Dictionary,
    PowerOfTwo,
    PowerOfTwoMinusOne,
}

struct SubstitutionConfiguration {
    substitution_by_zero_weight: usize,
    substitution_by_one_weight: usize,
    substitution_by_minus_one_weight: usize,
    substitution_from_dictionary_weight: usize,
    substitution_by_power_of_2_weight: usize,
    substitution_by_power_of_2_minus_one: usize,
    total_weight: usize,
}

/// Configuration for selecting a substitution mutation
impl SubstitutionConfiguration {
    pub fn new(
        substitution_by_zero_weight: usize,
        substitution_by_one_weight: usize,
        substitution_by_minus_one_weight: usize,
        substitution_from_dictionary_weight: usize,
        substitution_by_power_of_2: usize,
        substitution_by_power_of_2_minus_one: usize,
    ) -> Self {
        let total_weight = substitution_by_minus_one_weight
            + substitution_by_one_weight
            + substitution_by_zero_weight
            + substitution_from_dictionary_weight
            + substitution_by_power_of_2
            + substitution_by_power_of_2_minus_one;
        Self {
            substitution_by_zero_weight,
            substitution_by_one_weight,
            substitution_by_minus_one_weight,
            substitution_from_dictionary_weight,
            substitution_by_power_of_2_weight: substitution_by_power_of_2,
            substitution_by_power_of_2_minus_one,
            total_weight,
        }
    }

    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> SubstitutionMutation {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.substitution_by_zero_weight {
            return SubstitutionMutation::Zero;
        }
        selector -= self.substitution_by_zero_weight;
        if selector < self.substitution_by_one_weight {
            return SubstitutionMutation::One;
        }
        selector -= self.substitution_by_one_weight;
        if selector < self.substitution_by_minus_one_weight {
            return SubstitutionMutation::MinusOne;
        }
        selector -= self.substitution_by_minus_one_weight;
        if selector < self.substitution_from_dictionary_weight {
            return SubstitutionMutation::Dictionary;
        }
        selector -= self.substitution_from_dictionary_weight;
        if selector < self.substitution_by_power_of_2_weight {
            return SubstitutionMutation::PowerOfTwo;
        }
        return SubstitutionMutation::PowerOfTwoMinusOne;
    }
}

const BASIC_SUBSTITUTION_CONFIGURATION: SubstitutionConfiguration = SubstitutionConfiguration {
    substitution_by_zero_weight: 20,
    substitution_by_one_weight: 20,
    substitution_by_minus_one_weight: 20,
    substitution_from_dictionary_weight: 20,
    substitution_by_power_of_2_weight: 254,
    substitution_by_power_of_2_minus_one: 254,
    total_weight: 20 + 20 + 20 + 20 + 254 + 254,
};

enum InversionMutation {
    Additive,
    Multiplicative,
}
struct InversionConfiguration {
    additive_inversion_weight: usize,
    multiplicative_inversion_weight: usize,
    total_weight: usize,
}

impl InversionConfiguration {
    pub fn new(additive_inversion_weight: usize, multiplicative_inversion_weight: usize) -> Self {
        let total_weight = additive_inversion_weight + multiplicative_inversion_weight;
        Self { additive_inversion_weight, multiplicative_inversion_weight, total_weight }
    }
    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> InversionMutation {
        let selector = prng.gen_range(0..self.total_weight);
        if selector < self.additive_inversion_weight {
            return InversionMutation::Additive;
        }
        return InversionMutation::Multiplicative;
    }
}

const BASIC_INVERSION_CONFIGURATION: InversionConfiguration = InversionConfiguration {
    additive_inversion_weight: 10,
    multiplicative_inversion_weight: 1,
    total_weight: 10 + 1,
};

enum Pow2Update {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}
struct Pow2UpdateConfiguration {
    addition_weight: usize,
    subtraction_weight: usize,
    multiplication_weight: usize,
    division_weight: usize,
    total_weight: usize,
}

impl Pow2UpdateConfiguration {
    pub fn new(
        addition_weight: usize,
        subtraction_weight: usize,
        multiplication_weight: usize,
        division_weight: usize,
    ) -> Self {
        let total_weight =
            addition_weight + subtraction_weight + multiplication_weight + division_weight;
        Self {
            addition_weight,
            subtraction_weight,
            multiplication_weight,
            division_weight,
            total_weight,
        }
    }
    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> Pow2Update {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.addition_weight {
            return Pow2Update::Addition;
        }
        selector -= self.addition_weight;
        if selector < self.subtraction_weight {
            return Pow2Update::Subtraction;
        }
        selector -= self.subtraction_weight;
        if selector < self.multiplication_weight {
            return Pow2Update::Multiplication;
        }
        return Pow2Update::Division;
    }
}

const BASIC_POW_2_UPDATE_CONFIGURATION: Pow2UpdateConfiguration = Pow2UpdateConfiguration {
    addition_weight: 3,
    subtraction_weight: 3,
    multiplication_weight: 2,
    division_weight: 1,
    total_weight: 3 + 3 + 2 + 1,
};

enum SmallValueUpdate {
    Addition,
    Subtraction,
    Multiplication,
}
struct SmallValueUpdateConfiguration {
    addition_weight: usize,
    subtraction_weight: usize,
    multiplication_weight: usize,
    total_weight: usize,
}

impl SmallValueUpdateConfiguration {
    pub fn new(
        addition_weight: usize,
        subtraction_weight: usize,
        multiplication_weight: usize,
    ) -> Self {
        let total_weight = addition_weight + subtraction_weight + multiplication_weight;
        Self { addition_weight, subtraction_weight, multiplication_weight, total_weight }
    }
    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> SmallValueUpdate {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.addition_weight {
            return SmallValueUpdate::Addition;
        }
        selector -= self.addition_weight;
        if selector < self.subtraction_weight {
            return SmallValueUpdate::Subtraction;
        }
        return SmallValueUpdate::Multiplication;
    }
}

const BASIC_SMALL_VALUE_UPDATE_CONFIGURATION: SmallValueUpdateConfiguration =
    SmallValueUpdateConfiguration {
        addition_weight: 3,
        subtraction_weight: 3,
        multiplication_weight: 1,
        total_weight: 3 + 3 + 1,
    };

enum DictionaryUpdate {
    Addition,
    Subtraction,
    Multiplication,
}
struct DictionaryUpdateConfiguration {
    addition_weight: usize,
    subtraction_weight: usize,
    multiplication_weight: usize,
    total_weight: usize,
}

impl DictionaryUpdateConfiguration {
    pub fn new(
        addition_weight: usize,
        subtraction_weight: usize,
        multiplication_weight: usize,
    ) -> Self {
        let total_weight = addition_weight + subtraction_weight;
        Self { addition_weight, subtraction_weight, multiplication_weight, total_weight }
    }
    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> DictionaryUpdate {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.addition_weight {
            return DictionaryUpdate::Addition;
        }
        selector -= self.addition_weight;
        if selector < self.subtraction_weight {
            return DictionaryUpdate::Subtraction;
        }
        return DictionaryUpdate::Multiplication;
    }
}

const BASIC_DICTIONARY_UPDATE_CONFIGURATION: DictionaryUpdateConfiguration =
    DictionaryUpdateConfiguration {
        addition_weight: 2,
        subtraction_weight: 2,
        multiplication_weight: 1,
        total_weight: 2 + 2 + 1,
    };
enum TopLevelMutation {
    Substitution,
    Inversion,
    Pow2Update,
    SmallValueUpdate,
    DictionaryUpdate,
}
struct TopLevelMutationConfiguration {
    substitution_weight: usize,
    inversion_weight: usize,
    pow_2_update_weight: usize,
    small_value_update_weight: usize,
    dictionary_update_weight: usize,
    total_weight: usize,
}

impl TopLevelMutationConfiguration {
    pub fn new(
        substitution_weight: usize,
        inversion_weight: usize,
        pow_2_update_weight: usize,
        small_value_update_weight: usize,
        dictionary_update_weight: usize,
    ) -> Self {
        let total_weight = substitution_weight
            + inversion_weight
            + pow_2_update_weight
            + small_value_update_weight
            + dictionary_update_weight;
        Self {
            substitution_weight,
            inversion_weight,
            pow_2_update_weight,
            small_value_update_weight,
            dictionary_update_weight,
            total_weight,
        }
    }
    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> TopLevelMutation {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.substitution_weight {
            return TopLevelMutation::Substitution;
        }
        selector -= self.substitution_weight;
        if selector < self.inversion_weight {
            return TopLevelMutation::Inversion;
        }
        selector -= self.inversion_weight;
        if selector < self.pow_2_update_weight {
            return TopLevelMutation::Pow2Update;
        }
        selector -= self.pow_2_update_weight;
        if selector < self.small_value_update_weight {
            return TopLevelMutation::SmallValueUpdate;
        }
        return TopLevelMutation::DictionaryUpdate;
    }
}

const BASIC_TOPLEVEL_MUTATION_CONFIGURATION: TopLevelMutationConfiguration =
    TopLevelMutationConfiguration {
        substitution_weight: 10,
        inversion_weight: 1,
        pow_2_update_weight: 5,
        small_value_update_weight: 10,
        dictionary_update_weight: 10,
        total_weight: 10 + 1 + 5 + 10 + 10,
    };

struct FieldMutator<'a> {
    dictionary: &'a Vec<FieldElement>,
    prng: &'a mut XorShiftRng,
}

impl<'a> FieldMutator<'a> {
    pub fn new(dictionary: &'a Vec<FieldElement>, prng: &'a mut XorShiftRng) -> Self {
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
        match BASIC_SUBSTITUTION_CONFIGURATION.select(self.prng) {
            SubstitutionMutation::Zero => (FieldElement::from(0u32)),
            SubstitutionMutation::One => (FieldElement::from(1u32)),
            SubstitutionMutation::MinusOne => (-FieldElement::from(1u32)),
            SubstitutionMutation::Dictionary => (*self.dictionary.choose(self.prng).unwrap()),
            SubstitutionMutation::PowerOfTwo => unsafe {
                POWERS_OF_TWO.choose(self.prng).unwrap().clone()
            },
            SubstitutionMutation::PowerOfTwoMinusOne => unsafe {
                POWERS_OF_TWO_MINUS_ONE.choose(self.prng).unwrap().clone()
            },
        }
    }

    fn apply_inversion(&mut self, element: FieldElement) -> FieldElement {
        match BASIC_INVERSION_CONFIGURATION.select(self.prng) {
            InversionMutation::Additive => -element,
            InversionMutation::Multiplicative => element.inverse(),
        }
    }

    fn apply_pow_2_update(&mut self, element: FieldElement) -> FieldElement {
        let chosen_power_of_two = unsafe { POWERS_OF_TWO.choose(self.prng).unwrap() };
        let chosen_inverse_power_of_two =
            unsafe { INVERSE_POWERS_OF_TWO.choose(self.prng).unwrap() };
        match BASIC_POW_2_UPDATE_CONFIGURATION.select(self.prng) {
            Pow2Update::Addition => element + *chosen_power_of_two,
            Pow2Update::Subtraction => element - *chosen_power_of_two,
            Pow2Update::Multiplication => element * *chosen_power_of_two,
            Pow2Update::Division => element * *chosen_inverse_power_of_two,
        }
    }

    fn apply_small_value_update(&mut self, element: FieldElement) -> FieldElement {
        let small_value =
            FieldElement::from(self.prng.gen_range(SMALL_VALUE_MIN..=SMALL_VALUE_MAX));
        match BASIC_SMALL_VALUE_UPDATE_CONFIGURATION.select(self.prng) {
            SmallValueUpdate::Addition => element + small_value,
            SmallValueUpdate::Subtraction => element - small_value,
            SmallValueUpdate::Multiplication => element * small_value,
        }
    }

    fn apply_dictionary_update(&mut self, element: FieldElement) -> FieldElement {
        let dictionary_value = self.dictionary.choose(self.prng).unwrap();
        match BASIC_DICTIONARY_UPDATE_CONFIGURATION.select(self.prng) {
            DictionaryUpdate::Addition => element + *dictionary_value,
            DictionaryUpdate::Subtraction => element - *dictionary_value,
            DictionaryUpdate::Multiplication => element * *dictionary_value,
        }
    }

    pub fn mutate(&mut self, input: &InputValue) -> InputValue {
        let initial_field_value = match input {
            InputValue::Field(inner_field) => inner_field,
            _ => panic!("Shouldn't be used with other input value types"),
        }
        .clone();
        return InputValue::Field(match BASIC_TOPLEVEL_MUTATION_CONFIGURATION.select(self.prng) {
            TopLevelMutation::Substitution => self.apply_substitution(),
            TopLevelMutation::Inversion => self.apply_inversion(initial_field_value),
            TopLevelMutation::Pow2Update => self.apply_pow_2_update(initial_field_value),
            TopLevelMutation::SmallValueUpdate => {
                self.apply_small_value_update(initial_field_value)
            }
            TopLevelMutation::DictionaryUpdate => self.apply_dictionary_update(initial_field_value),
        });
    }
}
pub fn mutate_field_input_value(
    previous_input: &InputValue,
    dictionary: &Vec<FieldElement>,
    prng: &mut XorShiftRng,
) -> InputValue {
    let mut field_mutator = FieldMutator::new(&dictionary, prng);
    field_mutator.mutate(previous_input)
}
