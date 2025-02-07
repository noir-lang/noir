use crate::mutation::configurations::{
    TopLevelMutation, BASIC_SPLICE_MUTATION_CONFIGURATION, BASIC_TOP_LEVEL_MUTATION_CONFIGURATION,
};

use super::configurations::{
    ByteValueMutation, ByteValueMutationConfiguration, SpliceCandidate, SpliceMutation,
    StructuralMutation, BASIC_BYTE_VALUE_MUTATION_CONFIGURATION,
    BASIC_SPLICE_CANDIDATE_PRIORITIZATION_CONFIGURATION, BASIC_STRUCTURE_MUTATION_CONFIGURATION,
    DICTIONARY_EMPTY_BYTE_VALUE_MUTATION_CONFIGURATION,
};
use super::dictionary::IntDictionary;
use acvm::{AcirField, FieldElement};
use noirc_abi::input_parser::InputValue;
use rand::{seq::SliceRandom, Rng};
use rand_xorshift::XorShiftRng;
use std::cmp::min;

/// This file contains mechanisms for mutating string InputValues. It can perform the following mutations:
/// 1. Value mutations
///     a. Substitution a random character in the string with a random appropriate value from the dictionary
///     b. Substitution of a random character with a random character
/// 2. Structural mutations
///     a. Chaotically splicing the string with itself (constructing a new string from random chunks of initial string)
///     b. Duplication of a random chunk (picking a chunk of the string and inserting it several times)
///     c. Inserting a repeated random value
///     d. Swapping two chunks
///
/// It also contains the splicing mechanism used when splicing two inputs. It chooses between:
/// 1. Structured splicing (preserving the indices of the values)
/// 2. Chaotic (creating the new string from chunks of the two given strings)
///
const MIN_ASCII: u8 = 0x0;
const MAX_ASCII: u8 = 0x7f;

struct StringMutator<'a> {
    dictionary: &'a Vec<FieldElement>,
    prng: &'a mut XorShiftRng,
    value_mutation_configuration: &'static ByteValueMutationConfiguration,
}

impl<'a> StringMutator<'a> {
    pub fn new(dictionary: &'a IntDictionary, prng: &'a mut XorShiftRng) -> Self {
        let u8_dictionary = dictionary.get_dictionary_by_width(8);
        Self {
            dictionary: u8_dictionary,
            prng,
            value_mutation_configuration: if !u8_dictionary.is_empty() {
                &BASIC_BYTE_VALUE_MUTATION_CONFIGURATION
            } else {
                &DICTIONARY_EMPTY_BYTE_VALUE_MUTATION_CONFIGURATION
            },
        }
    }

    /// Perform one of value-changing mutations (substitution by a dictionary or random value)
    fn perform_value_mutation(&mut self, input: &Vec<u8>) -> Vec<u8> {
        let mut result = input.clone();
        let position = self.prng.gen_range(0..input.len());
        result[position] = match self.value_mutation_configuration.select(self.prng) {
            ByteValueMutation::DictionaryByte => {
                self.dictionary.choose(self.prng).unwrap().to_i128() as u8 & MAX_ASCII
            }
            ByteValueMutation::RandomByte => self.prng.gen_range(MIN_ASCII..=MAX_ASCII),
        };
        result
    }

    /// Mutate a string input value
    pub fn mutate(&mut self, input: &InputValue) -> InputValue {
        let mut result: Vec<u8> = match input {
            InputValue::String(inner_string) => inner_string,
            _ => panic!("Shouldn't be used with other input value types"),
        }
        .as_bytes()
        .to_vec();
        assert!(!result.is_empty());
        result = match BASIC_TOP_LEVEL_MUTATION_CONFIGURATION.select(self.prng) {
            TopLevelMutation::Value => self.perform_value_mutation(&result),
            TopLevelMutation::Structure => self.perform_structure_mutation(&result),
        };

        InputValue::String(
            String::from_utf8(result)
                .expect("We expect that the values in the string are just ASCII"),
        )
    }

    /// Perform one of structural mutations on the buffer
    fn perform_structure_mutation(&mut self, input_buffer: &Vec<u8>) -> Vec<u8> {
        match BASIC_STRUCTURE_MUTATION_CONFIGURATION.select(self.prng) {
            StructuralMutation::ChaoticSelfSplice => {
                self.chaotic_splice(input_buffer, input_buffer)
            }
            StructuralMutation::ChunkDuplication => self.duplicate_chunk(input_buffer),
            StructuralMutation::RandomValueDuplication => self.duplicate_random_value(input_buffer),
            StructuralMutation::Swap => self.swap(input_buffer),
        }
    }
    /// Swap 2 random chunks in the buffer
    fn swap(&mut self, buffer: &Vec<u8>) -> Vec<u8> {
        let mut result = Vec::new();
        let buffer_length = buffer.len();

        // We need to leave at least the last byte for the second chunk
        let first_chunk_position = self.prng.gen_range(0..(buffer_length - 1));

        // The second chunk starts after the first
        let second_chunk_position = self.prng.gen_range((first_chunk_position + 1)..buffer_length);

        let first_chunk_end =
            self.prng.gen_range((first_chunk_position + 1)..=second_chunk_position);

        let second_chunk_end = self.prng.gen_range((second_chunk_position + 1)..=buffer_length);

        // Leave the start in place
        result.extend_from_slice(&buffer[0..first_chunk_position]);

        // Insert second chunk
        result.extend_from_slice(&buffer[second_chunk_position..(second_chunk_end)]);

        // Insert what's in between the chunks
        result.extend_from_slice(&buffer[first_chunk_end..(second_chunk_position)]);

        // Insert first chunk
        result.extend_from_slice(&buffer[first_chunk_position..first_chunk_end]);

        // Insert the tail
        result.extend_from_slice(&buffer[second_chunk_end..buffer_length]);

        result
    }

    /// Take a random chunk of the input and insert it several times into the input
    fn duplicate_chunk(&mut self, input_buffer: &Vec<u8>) -> Vec<u8> {
        let mut result = input_buffer.clone();
        let buffer_length = input_buffer.len();
        // The maximum length of the chunk is half the total length
        let maximum_chunk_length = buffer_length / 2;

        // Get a random position for the chunk
        let chunk_position = self.prng.gen_range(0..=buffer_length - 1);

        // Pick size
        let chunk_size =
            self.prng.gen_range(1..=min(buffer_length - chunk_position, maximum_chunk_length));

        // Find an insertion position with enough space
        let insertion_position = self.prng.gen_range(0..(buffer_length - chunk_size));

        // Determine how many times to repeat
        let maximum_insertion_count = (buffer_length - insertion_position) / chunk_size;
        let insertion_count = self.prng.gen_range(0..=maximum_insertion_count);
        for i in 0..insertion_count {
            result.splice(
                (insertion_position + i * chunk_size)..(insertion_position + (i + 1) * chunk_size),
                input_buffer[chunk_position..(chunk_position + chunk_size)].iter().copied(),
            );
        }
        result
    }

    /// Take a random value and insert it several times
    fn duplicate_random_value(&mut self, input_buffer: &Vec<u8>) -> Vec<u8> {
        let mut result = input_buffer.clone();
        let buffer_length = input_buffer.len();

        // Find an insertion position with enough space
        let insertion_position = self.prng.gen_range(0..buffer_length);

        // Pick count
        let insertion_count = self.prng.gen_range(1..=(buffer_length - insertion_position));

        // Pick value
        let value = self.prng.gen_range(MIN_ASCII..=MAX_ASCII);
        for item in result.iter_mut().skip(insertion_position).take(insertion_count) {
            *item = value;
        }
        result
    }
    /// Create a spliced version of 2 buffers, where each element in the result is at the same index as in the original ones
    fn structured_splice(&mut self, first_buffer: &Vec<u8>, second_buffer: &Vec<u8>) -> Vec<u8> {
        let mut result = first_buffer.clone();
        let mut index = 0;
        let buffer_length = first_buffer.len();
        while index != buffer_length {
            // Pick the length of the sequence from 1 to maximum available
            let sequence_length = self.prng.gen_range(1..=(buffer_length - index));

            // If first buffer is selected for the chunk, do nothing (we already have that part in the result)
            // If the second is selected, copy the chunk into result
            match BASIC_SPLICE_CANDIDATE_PRIORITIZATION_CONFIGURATION.select(self.prng) {
                SpliceCandidate::First => {}
                SpliceCandidate::Second => {
                    result.splice(
                        index..(index + sequence_length),
                        second_buffer[index..(index + sequence_length)].iter().copied(),
                    );
                }
            }
            index += sequence_length;
        }
        result
    }

    /// Create buffer from random chunks of 2 buffers
    fn chaotic_splice(&mut self, first_buffer: &[u8], second_buffer: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        let mut index = 0;
        let buffer_length = first_buffer.len();
        result.resize(buffer_length, 0);
        while index != buffer_length {
            // Pick the length of the sequence from 1 to maximum available
            let sequence_length = self.prng.gen_range(1..=(buffer_length - index));

            let source_position = self.prng.gen_range(0..=(buffer_length - sequence_length));
            // If first buffer is selected for the chunk, do nothing (we already have that part in the result)
            // If the second is selected, copy the chunk into result
            match BASIC_SPLICE_CANDIDATE_PRIORITIZATION_CONFIGURATION.select(self.prng) {
                SpliceCandidate::First => {
                    result.splice(
                        index..(index + sequence_length),
                        first_buffer[source_position..(source_position + sequence_length)]
                            .iter()
                            .copied(),
                    );
                }
                SpliceCandidate::Second => {
                    result.splice(
                        index..(index + sequence_length),
                        second_buffer[source_position..(source_position + sequence_length)]
                            .iter()
                            .copied(),
                    );
                }
            }
            index += sequence_length;
        }
        result
    }

    /// Generate a combination of two string inputs
    pub fn splice(&mut self, first_input: &InputValue, second_input: &InputValue) -> InputValue {
        let first_buffer: Vec<u8> = match first_input {
            InputValue::String(inner_string) => inner_string,
            _ => panic!("Shouldn't be used with other input value types"),
        }
        .as_bytes()
        .to_vec();

        let second_buffer: Vec<u8> = match second_input {
            InputValue::String(inner_string) => inner_string,
            _ => panic!("Shouldn't be used with other input value types"),
        }
        .as_bytes()
        .to_vec();

        assert!(!first_buffer.is_empty());
        assert!(second_buffer.len() == first_buffer.len());

        let result = match BASIC_SPLICE_MUTATION_CONFIGURATION.select(self.prng) {
            SpliceMutation::PositionPreserving => {
                self.structured_splice(&first_buffer, &second_buffer)
            }
            SpliceMutation::RandomChunks => self.chaotic_splice(&first_buffer, &second_buffer),
        };
        InputValue::String(
            String::from_utf8(result)
                .expect("We expect that the values in the string are just ASCII"),
        )
    }
}

pub fn mutate_string_input_value(
    previous_input: &InputValue,
    prng: &mut XorShiftRng,
    dictionary: &IntDictionary,
) -> InputValue {
    let mut string_mutator = StringMutator::new(dictionary, prng);
    string_mutator.mutate(previous_input)
}

pub fn splice_string_input_value(
    first_input: &InputValue,
    second_input: &InputValue,
    prng: &mut XorShiftRng,
) -> InputValue {
    let dictionary = IntDictionary::default();
    let mut string_mutator = StringMutator::new(&dictionary, prng);
    string_mutator.splice(first_input, second_input)
}
