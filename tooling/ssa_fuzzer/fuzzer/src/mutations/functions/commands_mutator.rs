//! This file contains mechanisms for deterministically mutating a given vector of [FuzzerCommand](crate::fuzz_lib::base_context::FuzzerCommand) values

use crate::fuzz_lib::function_context::FuzzerFunctionCommand;
use crate::mutations::basic_types::vec::mutate_vec;
use crate::mutations::configuration::BASIC_VEC_MUTATION_CONFIGURATION;
use crate::mutations::functions::command::mutate_fuzzer_function_command;
use rand::rngs::StdRng;

pub(crate) fn mutate_vec_fuzzer_command(
    vec_fuzzer_command: &mut Vec<FuzzerFunctionCommand>,
    rng: &mut StdRng,
) {
    mutate_vec(
        vec_fuzzer_command,
        rng,
        mutate_fuzzer_function_command,
        BASIC_VEC_MUTATION_CONFIGURATION,
    );
}
