//! This file contains mechanisms for deterministically mutating a given [Argument](crate::fuzz_lib::instruction::Argument) value
//! Types of mutations applied:
//! 1. Random (randomly select a new argument value)
//! 2. Increment index
//! 3. Decrement index
//! 4. Change type

use crate::fuzz_lib::instruction::Argument;
use crate::mutations::{
    configuration::{ArgumentMutationOptions, BASIC_ARGUMENT_MUTATION_CONFIGURATION},
    instructions::type_mutations::type_mutator,
};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait ArgumentsMutator {
    fn mutate(rng: &mut StdRng, value: &mut Argument);
}

/// Return new random argument
struct RandomMutation;
impl ArgumentsMutator for RandomMutation {
    fn mutate(rng: &mut StdRng, value: &mut Argument) {
        let mut bytes = [0u8; 17];
        rng.fill(&mut bytes);
        *value = Unstructured::new(&bytes).arbitrary().unwrap();
    }
}

/// Increment index of the argument
struct IncrementArgumentIndexMutation;
impl ArgumentsMutator for IncrementArgumentIndexMutation {
    fn mutate(_rng: &mut StdRng, value: &mut Argument) {
        value.index = value.index.saturating_add(1);
    }
}

/// Decrement index of the argument
struct DecrementArgumentIndexMutation;
impl ArgumentsMutator for DecrementArgumentIndexMutation {
    fn mutate(_rng: &mut StdRng, value: &mut Argument) {
        value.index = value.index.saturating_sub(1);
    }
}

/// Change type of the argument
struct ChangeTypeMutation;
impl ArgumentsMutator for ChangeTypeMutation {
    fn mutate(rng: &mut StdRng, value: &mut Argument) {
        type_mutator(&mut value.value_type, rng);
    }
}

pub(crate) fn argument_mutator(argument: &mut Argument, rng: &mut StdRng) {
    match BASIC_ARGUMENT_MUTATION_CONFIGURATION.select(rng) {
        ArgumentMutationOptions::Random => RandomMutation::mutate(rng, argument),
        ArgumentMutationOptions::IncrementIndex => {
            IncrementArgumentIndexMutation::mutate(rng, argument)
        }
        ArgumentMutationOptions::DecrementIndex => {
            DecrementArgumentIndexMutation::mutate(rng, argument)
        }
        ArgumentMutationOptions::ChangeType => ChangeTypeMutation::mutate(rng, argument),
    }
}
