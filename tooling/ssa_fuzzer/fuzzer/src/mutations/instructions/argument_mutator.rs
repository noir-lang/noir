//! This file contains mechanisms for deterministically mutating a given [Argument](crate::fuzz_lib::instruction::Argument) value

use crate::fuzz_lib::instruction::NumericArgument;
use crate::mutations::{
    basic_types::{numeric_type::mutate_numeric_type, usize::mutate_usize},
    configuration::{
        BASIC_NUMERIC_ARGUMENT_MUTATION_CONFIGURATION, BASIC_NUMERIC_TYPE_MUTATION_CONFIGURATION,
        BASIC_USIZE_MUTATION_CONFIGURATION, NumericArgumentMutationOptions,
    },
};
use rand::rngs::StdRng;

pub(crate) fn numeric_argument_mutator(argument: &mut NumericArgument, rng: &mut StdRng) {
    match BASIC_NUMERIC_ARGUMENT_MUTATION_CONFIGURATION.select(rng) {
        NumericArgumentMutationOptions::MutateIndex => {
            mutate_usize(&mut argument.index, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
        }
        NumericArgumentMutationOptions::ChangeType => {
            mutate_numeric_type(
                &mut argument.numeric_type,
                rng,
                BASIC_NUMERIC_TYPE_MUTATION_CONFIGURATION,
            );
        }
    }
}
