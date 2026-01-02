use crate::fuzz_lib::initial_witness::WitnessValue;
use crate::mutations::basic_types::vec::mutate_vec;
use crate::mutations::configuration::{
    ARRAY_WITNESS_MUTATION_CONFIGURATION_INNER,
    DETERMINISTIC_NUMERIC_WITNESS_MUTATION_CONFIGURATION, NumericWitnessMutationConfig,
    VecMutationConfig,
};
use crate::mutations::initial_witness::numeric_witness;
use rand::rngs::StdRng;

pub(crate) fn generate_witness_of_the_same_type(
    rng: &mut StdRng,
    value: &WitnessValue,
) -> WitnessValue {
    match value {
        WitnessValue::Numeric(numeric) => {
            WitnessValue::Numeric(numeric_witness::generate_element_of_the_same_type(rng, *numeric))
        }
        WitnessValue::Array(array) => WitnessValue::Array(
            array.iter().map(|elem| generate_witness_of_the_same_type(rng, elem)).collect(),
        ),
    }
}

pub(crate) fn mutate(
    witness_value: &mut WitnessValue,
    rng: &mut StdRng,
    numeric_witness_mutation_config: NumericWitnessMutationConfig,
    array_witness_mutation_config: VecMutationConfig,
) {
    match witness_value {
        WitnessValue::Numeric(numeric) => {
            numeric_witness::mutate(numeric, rng, numeric_witness_mutation_config);
        }
        WitnessValue::Array(array) => {
            let element_type = array[0].clone();
            mutate_vec(
                array,
                rng,
                // if mutating inner array, use ARRAY_WITNESS_MUTATION_CONFIGURATION_INNER
                // in order not to remove/add the elements of the inner array
                // if for example initial_witness is [[1,2], [3, 4]] removing or adding something
                // to inner element will lead to different sizes and panic of fuzzer
                |elem, rng| {
                    mutate(
                        elem,
                        rng,
                        DETERMINISTIC_NUMERIC_WITNESS_MUTATION_CONFIGURATION,
                        ARRAY_WITNESS_MUTATION_CONFIGURATION_INNER,
                    );
                },
                |rng| generate_witness_of_the_same_type(rng, &element_type),
                array_witness_mutation_config,
            );
        }
    }
}
