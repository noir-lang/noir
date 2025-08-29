mod numeric_witness;
mod witness;

use crate::fuzz_lib::initial_witness::{WitnessValue, WitnessValueNumeric};
use crate::mutations::basic_types::vec::mutate_vec;
use crate::mutations::configuration::{
    BASIC_GENERATE_INITIAL_WITNESS_CONFIGURATION, BASIC_VEC_MUTATION_CONFIGURATION,
    GenerateInitialWitness, MAX_ARRAY_SIZE, SIZE_OF_SMALL_ARBITRARY_BUFFER,
};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

// TODO
fn generate_random_element_function(rng: &mut StdRng) -> WitnessValue {
    match BASIC_GENERATE_INITIAL_WITNESS_CONFIGURATION.select(rng) {
        GenerateInitialWitness::Numeric => {
            let mut bytes = [0u8; SIZE_OF_SMALL_ARBITRARY_BUFFER];
            rng.fill(&mut bytes);
            WitnessValue::Numeric(Unstructured::new(&bytes).arbitrary().unwrap())
        }
        // Choose type of the array element, now limited to numeric
        GenerateInitialWitness::Array => {
            let size = rng.gen_range(1..MAX_ARRAY_SIZE);
            let mut bytes = [0u8; SIZE_OF_SMALL_ARBITRARY_BUFFER];
            rng.fill(&mut bytes);
            let first_element: WitnessValueNumeric = Unstructured::new(&bytes).arbitrary().unwrap();
            let values = (0..size)
                .map(|_| {
                    WitnessValue::Numeric(numeric_witness::generate_element_of_the_same_type(
                        rng,
                        first_element,
                    ))
                })
                .collect();
            WitnessValue::Array(values)
        }
    }
}

pub(crate) fn mutate(witness_value: &mut Vec<WitnessValue>, rng: &mut StdRng) {
    mutate_vec(
        witness_value,
        rng,
        witness::mutate,
        generate_random_element_function,
        BASIC_VEC_MUTATION_CONFIGURATION,
    );
}
