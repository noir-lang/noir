use crate::mutations::{
    basic_types::numeric_type::generate_random_numeric_type,
    configuration::{
        BASIC_GENERATE_NUMERIC_TYPE_CONFIGURATION, BASIC_GENERATE_TYPE_CONFIGURATION, GenerateType,
        GenerateTypeConfig, MAX_ARRAY_SIZE,
    },
};
use noir_ssa_fuzzer::typed_value::Type;
use rand::{Rng, rngs::StdRng};
use std::sync::Arc;

fn generate_random_reference_type(
    rng: &mut StdRng,
    config: GenerateTypeConfig,
    vector_allowed: bool,
) -> Type {
    Type::Reference(Arc::new(generate_random_ssa_fuzzer_type_internal(rng, config, vector_allowed)))
}

fn generate_random_array_type(rng: &mut StdRng, config: GenerateTypeConfig) -> Type {
    Type::Array(
        Arc::new(vec![generate_random_ssa_fuzzer_type_internal(rng, config, false)]),
        rng.random_range(1..MAX_ARRAY_SIZE) as u32, // empty arrays are not allowed
    )
}

fn generate_random_vector_type(rng: &mut StdRng, config: GenerateTypeConfig) -> Type {
    Type::Vector(Arc::new(vec![generate_random_ssa_fuzzer_type_internal(rng, config, false)]))
}

fn generate_random_ssa_fuzzer_type_internal(
    rng: &mut StdRng,
    config: GenerateTypeConfig,
    vector_allowed: bool,
) -> Type {
    match config.select(rng) {
        GenerateType::Numeric => Type::Numeric(generate_random_numeric_type(
            rng,
            BASIC_GENERATE_NUMERIC_TYPE_CONFIGURATION,
        )),
        GenerateType::Reference => generate_random_reference_type(rng, config, vector_allowed),
        GenerateType::Array => generate_random_array_type(rng, config),
        GenerateType::Vector if vector_allowed => generate_random_vector_type(rng, config),
        GenerateType::Vector => generate_random_ssa_fuzzer_type_internal(rng, config, false),
    }
}

pub(crate) fn generate_random_ssa_fuzzer_type(
    rng: &mut StdRng,
    config: GenerateTypeConfig,
) -> Type {
    generate_random_ssa_fuzzer_type_internal(rng, config, true)
}

pub(crate) fn mutate_ssa_fuzzer_type(type_: &mut Type, rng: &mut StdRng) {
    *type_ = generate_random_ssa_fuzzer_type(rng, BASIC_GENERATE_TYPE_CONFIGURATION);
}
