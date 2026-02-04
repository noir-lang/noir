mod numeric_witness;
mod witness;

use crate::fuzz_lib::initial_witness::WitnessValue;
use crate::mutations::basic_types::vec::mutate_vec;
use crate::mutations::configuration::{
    ARRAY_WITNESS_MUTATION_CONFIGURATION, BASIC_GENERATE_INITIAL_WITNESS_CONFIGURATION,
    BASIC_NUMERIC_WITNESS_MUTATION_CONFIGURATION, BASIC_VEC_MUTATION_CONFIGURATION,
    GenerateInitialWitness, MAX_ARRAY_SIZE, SIZE_OF_SMALL_ARBITRARY_BUFFER,
};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

fn generate_random_witness_value(rng: &mut StdRng) -> WitnessValue {
    match BASIC_GENERATE_INITIAL_WITNESS_CONFIGURATION.select(rng) {
        GenerateInitialWitness::Numeric => {
            let mut bytes = [0u8; SIZE_OF_SMALL_ARBITRARY_BUFFER];
            rng.fill(&mut bytes);
            WitnessValue::Numeric(Unstructured::new(&bytes).arbitrary().unwrap())
        }
        GenerateInitialWitness::Array => {
            let size = rng.random_range(1..MAX_ARRAY_SIZE);
            let first_element = generate_random_witness_value(rng);
            let values = (0..size)
                .map(|_| witness::generate_witness_of_the_same_type(rng, &first_element))
                .collect();
            WitnessValue::Array(values)
        }
    }
}

pub(crate) fn mutate(witness_value: &mut Vec<WitnessValue>, rng: &mut StdRng) {
    mutate_vec(
        witness_value,
        rng,
        |elem, rng| {
            witness::mutate(
                elem,
                rng,
                BASIC_NUMERIC_WITNESS_MUTATION_CONFIGURATION,
                ARRAY_WITNESS_MUTATION_CONFIGURATION,
            );
        },
        generate_random_witness_value,
        BASIC_VEC_MUTATION_CONFIGURATION,
    );
}
