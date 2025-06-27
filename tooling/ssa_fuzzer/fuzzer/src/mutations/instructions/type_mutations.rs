//! This file contains mechanisms for deterministically mutating a given [ValueType](noir_ssa_fuzzer::typed_value::ValueType) value

use crate::mutations::configuration::{
    BASIC_VALUE_TYPE_MUTATION_CONFIGURATION, ValueTypeMutationOptions,
};
use libfuzzer_sys::arbitrary::Unstructured;
use noir_ssa_fuzzer::typed_value::ValueType;
use rand::{Rng, rngs::StdRng};

trait TypeMutator {
    fn mutate(rng: &mut StdRng, value: &mut ValueType);
}

struct RandomMutation;
impl TypeMutator for RandomMutation {
    fn mutate(rng: &mut StdRng, value: &mut ValueType) {
        let mut bytes = [0u8; 17];
        rng.fill(&mut bytes);
        *value = Unstructured::new(&bytes).arbitrary().unwrap();
    }
}

pub(crate) fn type_mutator(value_type: &mut ValueType, rng: &mut StdRng) {
    match BASIC_VALUE_TYPE_MUTATION_CONFIGURATION.select(rng) {
        ValueTypeMutationOptions::Random => RandomMutation::mutate(rng, value_type),
    }
}
