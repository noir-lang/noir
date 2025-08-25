//! This file contains mechanisms for deterministically mutating a given [Argument](crate::fuzz_lib::instruction::Argument) value

use crate::fuzz_lib::instruction::Argument;
use crate::mutations::{
    basic_types::{numeric_type::mutate_numeric_type, usize::mutate_usize},
    configuration::{
        ArgumentMutationOptions, BASIC_ARGUMENT_MUTATION_CONFIGURATION,
        BASIC_NUMERIC_TYPE_MUTATION_CONFIGURATION, BASIC_USIZE_MUTATION_CONFIGURATION,
    },
};
use rand::rngs::StdRng;

pub(crate) fn argument_mutator(argument: &mut Argument, rng: &mut StdRng) {
    match BASIC_ARGUMENT_MUTATION_CONFIGURATION.select(rng) {
        ArgumentMutationOptions::MutateIndex => {
            mutate_usize(&mut argument.index, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
        }
        ArgumentMutationOptions::ChangeType => {
            mutate_numeric_type(
                &mut argument.value_type,
                rng,
                BASIC_NUMERIC_TYPE_MUTATION_CONFIGURATION,
            );
        }
    }
}
