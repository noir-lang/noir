mod command;
mod commands_mutator;
mod function;
mod input_types;

use crate::fuzz_lib::function_context::FunctionData;
use crate::mutations::{
    basic_types::{ssa_fuzzer_type::generate_random_ssa_fuzzer_type, vec::mutate_vec},
    configuration::{BASIC_GENERATE_TYPE_CONFIGURATION, BASIC_VEC_MUTATION_CONFIGURATION},
    functions::commands_mutator::generate_random_fuzzer_function_command,
    functions::function::mutate_function,
};
use rand::{Rng, rngs::StdRng};

fn generate_random_function_data(rng: &mut StdRng) -> FunctionData {
    FunctionData {
        commands: vec![generate_random_fuzzer_function_command(rng)],
        return_instruction_block_idx: rng.gen_range(u8::MIN..u8::MAX).into(),
        return_type: generate_random_ssa_fuzzer_type(rng, BASIC_GENERATE_TYPE_CONFIGURATION),
        input_types: vec![generate_random_ssa_fuzzer_type(rng, BASIC_GENERATE_TYPE_CONFIGURATION)],
    }
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
