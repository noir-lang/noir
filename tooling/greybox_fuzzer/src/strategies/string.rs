use acvm::AcirField;
use noirc_abi::input_parser::InputValue;
use rand::{seq::SliceRandom, Rng};
use rand_xorshift::XorShiftRng;

use super::int::IntDictionary;

const RANDOM_BYTE_MUTATION_WEIGHT: usize = 0x40;
const DICTIONARY_BYTE_MUTATION_WEIGHT: usize = 0xC0;
const TOTAL_WEIGHT: usize = RANDOM_BYTE_MUTATION_WEIGHT + DICTIONARY_BYTE_MUTATION_WEIGHT;
pub fn mutate_string_input_value(
    previous_input: &InputValue,
    prng: &mut XorShiftRng,
    dictionary: &IntDictionary,
) -> InputValue {
    let mut initial_bytes: Vec<u8> = match previous_input {
        InputValue::String(inner_string) => inner_string,
        _ => panic!("Shouldn't be used with other input value types"),
    }
    .as_bytes()
    .to_vec();
    assert!(initial_bytes.len() != 0);
    let mut chosen_mutation = prng.gen_range(0..TOTAL_WEIGHT);
    let position = prng.gen_range(0..initial_bytes.len());
    if chosen_mutation < DICTIONARY_BYTE_MUTATION_WEIGHT {
        let char_dictionary = &dictionary.get_dictionary_by_width(8);
        if char_dictionary.len() == 0 {
            initial_bytes[position] = prng.gen_range(0..0x7f);
        }
        initial_bytes[position] = char_dictionary.choose(prng).unwrap().to_i128() as u8;
        return InputValue::String(
            String::from_utf8(initial_bytes)
                .expect("We expect that the values in the string are just ASCII"),
        );
    }
    chosen_mutation -= DICTIONARY_BYTE_MUTATION_WEIGHT;
    assert!(chosen_mutation < RANDOM_BYTE_MUTATION_WEIGHT);
    initial_bytes[position] = prng.gen_range(0..0x7f);
    InputValue::String(
        String::from_utf8(initial_bytes)
            .expect("We expect that the values in the string are just ASCII"),
    )
}

pub fn splice_string_input_value(
    first_input: &InputValue,
    second_input: &InputValue,
    prng: &mut XorShiftRng,
) -> InputValue {
    let mut first_initial_bytes: Vec<u8> = match first_input {
        InputValue::String(inner_string) => inner_string,
        _ => panic!("Shouldn't be used with other input value types"),
    }
    .as_bytes()
    .to_vec();
    let second_initial_bytes: Vec<u8> = match second_input {
        InputValue::String(inner_string) => inner_string,
        _ => panic!("Shouldn't be used with other input value types"),
    }
    .as_bytes()
    .to_vec();
    assert!(!first_initial_bytes.is_empty());
    assert!(second_initial_bytes.len() == first_initial_bytes.len());
    let mut index = 0;
    while index != first_initial_bytes.len() {
        let sequence_length = prng.gen_range(1..=(first_initial_bytes.len() - index));
        if prng.gen_bool(0.5) {
            first_initial_bytes.splice(
                index..(index + sequence_length),
                second_initial_bytes[index..(index + sequence_length)].iter().copied(),
            );
        }
        index += sequence_length;
    }
    return InputValue::String(
        String::from_utf8(first_initial_bytes)
            .expect("We expect that the values in the string are just ASCII"),
    );
}
