use crate::fuzz_lib::fuzz_target_lib::{FieldRepresentation, WitnessValue};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait WitnessMutator {
    fn mutate(&self, rng: &mut StdRng, value: WitnessValue) -> WitnessValue;
}
trait WitnessMutatorFactory {
    fn new() -> Box<dyn WitnessMutator>;
}

struct RandomMutation;
impl WitnessMutator for RandomMutation {
    fn mutate(&self, rng: &mut StdRng, _value: WitnessValue) -> WitnessValue {
        let mut bytes = [0u8; 17];
        rng.fill(&mut bytes);
        return Unstructured::new(&bytes).arbitrary().unwrap();
    }
}
impl WitnessMutatorFactory for RandomMutation {
    fn new() -> Box<dyn WitnessMutator> {
        Box::new(RandomMutation)
    }
}

struct MaxValueMutation;
impl WitnessMutator for MaxValueMutation {
    fn mutate(&self, rng: &mut StdRng, value: WitnessValue) -> WitnessValue {
        match value {
            WitnessValue::Field(_) => WitnessValue::Field(FieldRepresentation {
                high: 64323764613183177041862057485226039389,
                low: 53438638232309528389504892708671455232, // high * 2^128 + low = p - 1
            }),
            WitnessValue::U64(_) => WitnessValue::U64(u64::MAX),
            WitnessValue::Boolean(_) => WitnessValue::Boolean(true),
            WitnessValue::I64(_) => WitnessValue::I64(9223372036854775807), // 2^63 - 1
            WitnessValue::I32(_) => WitnessValue::I32(2147483647),          // 2^31 - 1
        }
    }
}
impl WitnessMutatorFactory for MaxValueMutation {
    fn new() -> Box<dyn WitnessMutator> {
        Box::new(MaxValueMutation)
    }
}

struct MinValueMutation;
impl WitnessMutator for MinValueMutation {
    fn mutate(&self, rng: &mut StdRng, value: WitnessValue) -> WitnessValue {
        match value {
            WitnessValue::Field(_) => WitnessValue::Field(FieldRepresentation { high: 0, low: 0 }),
            WitnessValue::U64(_) => WitnessValue::U64(0),
            WitnessValue::Boolean(_) => WitnessValue::Boolean(false),
            WitnessValue::I64(_) => WitnessValue::I64(1 << 63),
            WitnessValue::I32(_) => WitnessValue::I32(1 << 31),
        }
    }
}
impl WitnessMutatorFactory for MinValueMutation {
    fn new() -> Box<dyn WitnessMutator> {
        Box::new(MinValueMutation)
    }
}

struct DefaultMutation;
impl WitnessMutator for DefaultMutation {
    fn mutate(&self, rng: &mut StdRng, value: WitnessValue) -> WitnessValue {
        value
    }
}
impl WitnessMutatorFactory for DefaultMutation {
    fn new() -> Box<dyn WitnessMutator> {
        Box::new(DefaultMutation)
    }
}

fn mutation_factory(rng: &mut StdRng) -> Box<dyn WitnessMutator> {
    let mutator = if rng.gen_bool(0.5) {
        RandomMutation::new()
    } else if rng.gen_bool(0.3) {
        MaxValueMutation::new()
    } else if rng.gen_bool(0.2) {
        MinValueMutation::new()
    } else {
        DefaultMutation::new()
    };
    mutator
}

pub(crate) fn witness_mutator(
    witness_value: Vec<WitnessValue>,
    rng: &mut StdRng,
) -> Vec<WitnessValue> {
    let mut witness_values: Vec<WitnessValue> = Vec::new();
    for witness in witness_value {
        let mutator = mutation_factory(rng);
        let witness = mutator.mutate(rng, witness);
        witness_values.push(witness);
    }
    witness_values
}
