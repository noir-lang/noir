use std::cmp::min;

use noirc_abi::input_parser::InputValue;
use rand::Rng;
use rand_xorshift::XorShiftRng;

use crate::mutation::configurations::{BASIC_SPLICE_MUTATION_CONFIGURATION, SpliceMutationOptions};

use super::configurations::{
    BASIC_SPLICE_CANDIDATE_PRIORITIZATION_CONFIGURATION,
    BASIC_VECTOR_STRUCTURE_MUTATION_CONFIGURATION, SpliceCandidateOptions,
    StructuralMutationOptions,
};
struct ArrayMutator<'a> {
    prng: &'a mut XorShiftRng,
}

impl<'a> ArrayMutator<'a> {
    pub fn new(prng: &'a mut XorShiftRng) -> Self {
        Self { prng }
    }

    /// Create a spliced version of 2 buffers, where each element in the result is at the same index as in the original ones
    fn structured_splice(
        &mut self,
        first_buffer: &[InputValue],
        second_buffer: &[InputValue],
    ) -> Vec<InputValue> {
        let mut result = first_buffer.to_vec();
        let mut index = 0;
        let buffer_length = first_buffer.len();
        while index != buffer_length {
            // Pick the length of the sequence from 1 to maximum available
            let sequence_length = self.prng.random_range(1..=(buffer_length - index));

            // If first buffer is selected for the chunk, do nothing (we already have that part in the result)
            // If the second is selected, copy the chunk into result
            match BASIC_SPLICE_CANDIDATE_PRIORITIZATION_CONFIGURATION.select(self.prng) {
                SpliceCandidateOptions::First => {}
                SpliceCandidateOptions::Second => {
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
        first_buffer: &[InputValue],
        second_buffer: &[InputValue],
    ) -> Vec<InputValue> {
        let mut result = Vec::new();
        let mut index = 0;
        let buffer_length = first_buffer.len();
        while index != buffer_length {
            // Pick the length of the sequence from 1 to maximum available
            let sequence_length = self.prng.random_range(1..=(buffer_length - index));

            let source_position = self.prng.random_range(0..=(buffer_length - sequence_length));
            // If first buffer is selected for the chunk, do nothing (we already have that part in the result)
            // If the second is selected, copy the chunk into result
            match BASIC_SPLICE_CANDIDATE_PRIORITIZATION_CONFIGURATION.select(self.prng) {
                SpliceCandidateOptions::First => {
                    result.extend_from_slice(
                        &first_buffer[source_position..(source_position + sequence_length)],
                    );
                }
                SpliceCandidateOptions::Second => {
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
            SpliceMutationOptions::PositionPreserving => {
                self.structured_splice(first_buffer, second_buffer)
            }
            SpliceMutationOptions::RandomChunks => self.chaotic_splice(first_buffer, second_buffer),
        };
        InputValue::Vec(result)
    }

    /// Perform one of structural mutations on the vector of input values
    pub fn perform_structure_mutation_on_vector(
        &mut self,
        input_buffer: &[InputValue],
    ) -> Vec<InputValue> {
        match BASIC_VECTOR_STRUCTURE_MUTATION_CONFIGURATION.select(self.prng) {
            StructuralMutationOptions::ChaoticSelfSplice => {
                self.chaotic_splice(input_buffer, input_buffer)
            }
            StructuralMutationOptions::ChunkDuplication => self.duplicate_chunk(input_buffer),
            StructuralMutationOptions::Swap => self.swap(input_buffer),
            StructuralMutationOptions::RandomValueDuplication => {
                panic!("Vector mutations should have a value duplication weight of zero")
            }
        }
    }

    /// Swap 2 random chunks in the buffer
    fn swap(&mut self, buffer: &[InputValue]) -> Vec<InputValue> {
        let mut result = Vec::new();
        let buffer_length = buffer.len();

        // We need to leave at least the last byte for the second chunk
        let first_chunk_position = self.prng.random_range(0..(buffer_length - 1));

        // The second chunk starts after the first
        let second_chunk_position =
            self.prng.random_range((first_chunk_position + 1)..buffer_length);

        let first_chunk_end =
            self.prng.random_range((first_chunk_position + 1)..=second_chunk_position);

        let second_chunk_end = self.prng.random_range((second_chunk_position + 1)..=buffer_length);

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
    fn duplicate_chunk(&mut self, input_buffer: &[InputValue]) -> Vec<InputValue> {
        let mut result = input_buffer.to_vec();
        let buffer_length = input_buffer.len();
        // The maximum length of the chunk is half the total length
        let maximum_chunk_length = buffer_length / 2;

        // Get a random position for the chunk
        let chunk_position = self.prng.random_range(0..=buffer_length - 1);

        // Pick size
        let chunk_size =
            self.prng.random_range(1..=min(buffer_length - chunk_position, maximum_chunk_length));

        // Find an insertion position with enough space
        let insertion_position = self.prng.random_range(0..(buffer_length - chunk_size));

        // Determine how many times to repeat
        let maximum_insertion_count = (buffer_length - insertion_position) / chunk_size;
        let insertion_count = self.prng.random_range(0..=maximum_insertion_count);
        for i in 0..insertion_count {
            result.splice(
                (insertion_position + i * chunk_size)..(insertion_position + (i + 1) * chunk_size),
                input_buffer[chunk_position..(chunk_position + chunk_size)].iter().cloned(),
            );
        }
        result
    }
}

pub fn splice_array_structure(
    first_input: &InputValue,
    second_input: &InputValue,
    prng: &mut XorShiftRng,
) -> InputValue {
    let mut array_splicer = ArrayMutator::new(prng);
    array_splicer.splice(first_input, second_input)
}

pub fn mutate_vector_structure(input: &[InputValue], prng: &mut XorShiftRng) -> Vec<InputValue> {
    let mut array_mutator = ArrayMutator::new(prng);
    array_mutator.perform_structure_mutation_on_vector(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use acvm::{AcirField, FieldElement};
    use noirc_abi::input_parser::InputValue;
    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;

    #[test]
    fn test_swap() {
        // Create a deterministic PRNG for testing
        let seed = [42u8; 16];
        let mut prng = XorShiftRng::from_seed(seed);

        // Create a test array mutator
        let mut array_mutator = ArrayMutator::new(&mut prng);

        // Create a test buffer with distinct values for easy verification
        let buffer: Vec<InputValue> =
            (0..10).map(|i| InputValue::Field(FieldElement::from(i as u128))).collect();

        // Perform the swap operation
        let result = array_mutator.swap(&buffer);

        // Verify the result has the same length as the input
        assert_eq!(result.len(), buffer.len());

        // Verify that all elements from the original buffer are present in the result
        // (though potentially in a different order)
        let mut original_elements: Vec<u128> = buffer
            .iter()
            .map(|v| match v {
                InputValue::Field(i) => i.to_u128(),
                _ => panic!("Unexpected input value type"),
            })
            .collect();

        let mut result_elements: Vec<u128> = result
            .iter()
            .map(|v| match v {
                InputValue::Field(i) => i.to_u128(),
                _ => panic!("Unexpected input value type"),
            })
            .collect();

        original_elements.sort();
        result_elements.sort();

        assert_eq!(original_elements, result_elements);

        // Verify that the order has changed (the swap actually did something)
        // This might rarely fail if the random swap happens to produce the same order,
        // but with our seed it should be consistent
        assert_ne!(
            buffer
                .iter()
                .map(|v| match v {
                    InputValue::Field(i) => i.to_u128(),
                    _ => panic!("Unexpected input value type"),
                })
                .collect::<Vec<_>>(),
            result
                .iter()
                .map(|v| match v {
                    InputValue::Field(i) => i.to_u128(),
                    _ => panic!("Unexpected input value type"),
                })
                .collect::<Vec<_>>()
        );
    }
}
