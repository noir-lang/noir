//! This file contains mechanisms for deterministically mutating a given [WitnessValue](crate::fuzz_lib::fuzz_target_lib::WitnessValue) value
//! Types of mutations applied:
//! 1. Random (randomly select a new witness value)
//! 2. Max value
//! 3. Min value

use crate::fuzz_lib::fuzz_target_lib::{FieldRepresentation, WitnessValue};
use crate::mutations::configuration::{
    BASIC_WITNESS_MUTATION_CONFIGURATION, WitnessMutationOptions,
};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait WitnessMutator {
    fn mutate(&self, rng: &mut StdRng, value: &WitnessValue) -> WitnessValue;
}
trait WitnessMutatorFactory {
    fn new_box() -> Box<dyn WitnessMutator>;
}

/// Return new random witness value
#[derive(Default)]
struct RandomMutation;
impl WitnessMutator for RandomMutation {
    fn mutate(&self, rng: &mut StdRng, _value: &WitnessValue) -> WitnessValue {
        let mut bytes = [0u8; 17];
        rng.fill(&mut bytes);
        Unstructured::new(&bytes).arbitrary().unwrap()
    }
}

/// Return witness value with max value
#[derive(Default)]
struct MaxValueMutation;
impl WitnessMutator for MaxValueMutation {
    fn mutate(&self, _rng: &mut StdRng, value: &WitnessValue) -> WitnessValue {
        match value {
            WitnessValue::Field(_) => WitnessValue::Field(FieldRepresentation {
                high: 64323764613183177041862057485226039389,
                low: 53438638232309528389504892708671455232, // high * 2^128 + low = p - 1
            }),
            WitnessValue::U64(_) => WitnessValue::U64(u64::MAX),
            WitnessValue::Boolean(_) => WitnessValue::Boolean(true),
            WitnessValue::I64(_) => WitnessValue::I64((1 << 63) - 1), // 2^63 - 1, sign bit is 0
            WitnessValue::I32(_) => WitnessValue::I32((1 << 31) - 1), // 2^31 - 1, sign bit is 0
        }
    }
}

/// Return witness value with min value
#[derive(Default)]
struct MinValueMutation;
impl WitnessMutator for MinValueMutation {
    fn mutate(&self, _rng: &mut StdRng, value: &WitnessValue) -> WitnessValue {
        match value {
            WitnessValue::Field(_) => WitnessValue::Field(FieldRepresentation { high: 0, low: 0 }),
            WitnessValue::U64(_) => WitnessValue::U64(0),
            WitnessValue::Boolean(_) => WitnessValue::Boolean(false),
            WitnessValue::I64(_) => WitnessValue::I64(1 << 63), // 2^63, sign bit is 1
            WitnessValue::I32(_) => WitnessValue::I32(1 << 31), // 2^31, sign bit is 1
        }
    }
}

impl<T> WitnessMutatorFactory for T
where
    T: WitnessMutator + Default + 'static,
{
    fn new_box() -> Box<dyn WitnessMutator> {
        Box::new(T::default())
    }
}

fn mutation_factory(rng: &mut StdRng) -> Box<dyn WitnessMutator> {
    match BASIC_WITNESS_MUTATION_CONFIGURATION.select(rng) {
        WitnessMutationOptions::Random => RandomMutation::new_box(),
        WitnessMutationOptions::MaxValue => MaxValueMutation::new_box(),
        WitnessMutationOptions::MinValue => MinValueMutation::new_box(),
    }
}

pub(crate) fn witness_mutate(witness_value: &WitnessValue, rng: &mut StdRng) -> WitnessValue {
    let mutator = mutation_factory(rng);
    mutator.mutate(rng, witness_value)
}
