mod command;
mod commands_mutator;
mod function;

use crate::fuzz_lib::function_context::FunctionData;
use crate::mutations::{
    basic_types::vec::mutate_vec, configuration::BASIC_VEC_MUTATION_CONFIGURATION,
    functions::function::mutate_function,
};
use rand::rngs::StdRng;

pub(crate) fn mutate(vec_function_data: &mut Vec<FunctionData>, rng: &mut StdRng) {
    mutate_vec(vec_function_data, rng, mutate_function, BASIC_VEC_MUTATION_CONFIGURATION);
}
