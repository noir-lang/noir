mod command;
mod commands_mutator;
mod function;

use crate::fuzz_lib::function_context::FunctionData;
use crate::mutations::{
    basic_types::vec::mutate_vec,
    configuration::{BASIC_VEC_MUTATION_CONFIGURATION, SIZE_OF_LARGE_ARBITRARY_BUFFER},
    functions::function::mutate_function,
};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

fn generate_random_function_data(rng: &mut StdRng) -> FunctionData {
    let mut buf = [0u8; SIZE_OF_LARGE_ARBITRARY_BUFFER];
    rng.fill(&mut buf);
    let mut unstructured = Unstructured::new(&buf);
    unstructured.arbitrary().unwrap()
}

pub(crate) fn mutate(vec_function_data: &mut Vec<FunctionData>, rng: &mut StdRng) {
    mutate_vec(
        vec_function_data,
        rng,
        mutate_function,
        generate_random_function_data,
        BASIC_VEC_MUTATION_CONFIGURATION,
    );
}
