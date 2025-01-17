use noirc_abi::input_parser::InputValue;
use rand::Rng;
use rand_xorshift::XorShiftRng;

use crate::mutation::configurations::{SpliceMutation, BASIC_SPLICE_MUTATION_CONFIGURATION};

use super::configurations::{SpliceCandidate, BASIC_SPLICE_CANDIDATE_PRIORITIZATION_CONFIGURATION};
struct ArraySplicer<'a> {
    prng: &'a mut XorShiftRng,
}

impl<'a> ArraySplicer<'a> {
    pub fn new(prng: &'a mut XorShiftRng) -> Self {
        Self { prng }
    }

    /// Create a spliced version of 2 buffers, where each element in the result is at the same index as in the original ones
    fn structured_splice(
        &mut self,
        first_buffer: &Vec<InputValue>,
        second_buffer: &Vec<InputValue>,
    ) -> Vec<InputValue> {
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
                        second_buffer[index..(index + sequence_length)].iter().cloned(),
                    );
                }
            }
            index += sequence_length;
        }
        result
    }

    /// Create buffer from random chunks of 2 buffers
    fn chaotic_splice(
        &mut self,
        first_buffer: &Vec<InputValue>,
        second_buffer: &Vec<InputValue>,
    ) -> Vec<InputValue> {
        let mut result = Vec::new();
        let mut index = 0;
        let buffer_length = first_buffer.len();
        while index != buffer_length {
            // Pick the length of the sequence from 1 to maximum available
            let sequence_length = self.prng.gen_range(1..=(buffer_length - index));

            let source_position = self.prng.gen_range(0..=(buffer_length - sequence_length));
            // If first buffer is selected for the chunk, do nothing (we already have that part in the result)
            // If the second is selected, copy the chunk into result
            match BASIC_SPLICE_CANDIDATE_PRIORITIZATION_CONFIGURATION.select(self.prng) {
                SpliceCandidate::First => {
                    result.extend_from_slice(
                        &first_buffer[source_position..(source_position + sequence_length)],
                    );
                }
                SpliceCandidate::Second => {
                    result.extend_from_slice(
                        &second_buffer[source_position..(source_position + sequence_length)],
                    );
                }
            }
            index += sequence_length;
        }
        result
    }

    /// Generate a combination of two string inputs
    pub fn splice(&mut self, first_input: &InputValue, second_input: &InputValue) -> InputValue {
        let first_buffer = match first_input {
            InputValue::Vec(inner_vec) => inner_vec,
            _ => panic!("Shouldn't be used with other input value types"),
        };

        let second_buffer = match second_input {
            InputValue::Vec(inner_vec) => inner_vec,
            _ => panic!("Shouldn't be used with other input value types"),
        };

        assert!(!first_buffer.is_empty());
        assert!(second_buffer.len() == first_buffer.len());

        let result = match BASIC_SPLICE_MUTATION_CONFIGURATION.select(self.prng) {
            SpliceMutation::PositionPreserving => {
                self.structured_splice(&first_buffer, &second_buffer)
            }
            SpliceMutation::RandomChunks => self.chaotic_splice(&first_buffer, &second_buffer),
        };
        InputValue::Vec(result)
    }
}

pub fn splice_array_structure(
    first_input: &InputValue,
    second_input: &InputValue,
    prng: &mut XorShiftRng,
) -> InputValue {
    let mut array_splicer = ArraySplicer::new(prng);
    array_splicer.splice(first_input, second_input)
}
