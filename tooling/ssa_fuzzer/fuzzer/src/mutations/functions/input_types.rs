use crate::mutations::{
    basic_types::{
        ssa_fuzzer_type::{generate_random_ssa_fuzzer_type, mutate_ssa_fuzzer_type},
        vec::mutate_vec,
    },
    configuration::{BASIC_GENERATE_TYPE_CONFIGURATION, BASIC_VEC_MUTATION_CONFIGURATION},
};
use noir_ssa_fuzzer::typed_value::Type;
use rand::rngs::StdRng;

pub(crate) fn generate_input_type(rng: &mut StdRng) -> Type {
    generate_random_ssa_fuzzer_type(rng, BASIC_GENERATE_TYPE_CONFIGURATION)
}

pub(crate) fn mutate_input_types(input_types: &mut Vec<Type>, rng: &mut StdRng) {
    mutate_vec(
        input_types,
        rng,
        mutate_ssa_fuzzer_type,
        generate_input_type,
        BASIC_VEC_MUTATION_CONFIGURATION,
    );
}
