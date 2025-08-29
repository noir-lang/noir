use crate::fuzz_lib::initial_witness::WitnessValue;
use crate::mutations::configuration::BASIC_WITNESS_MUTATION_CONFIGURATION;
use crate::mutations::initial_witness::numeric_witness;
use rand::rngs::StdRng;

pub(crate) fn mutate(witness_value: &mut WitnessValue, rng: &mut StdRng) {
    match witness_value {
        WitnessValue::Numeric(numeric) => {
            numeric_witness::mutate(numeric, rng, BASIC_WITNESS_MUTATION_CONFIGURATION);
        }
        WitnessValue::Array(_array) => {
            // TODO
        }
    }
}
