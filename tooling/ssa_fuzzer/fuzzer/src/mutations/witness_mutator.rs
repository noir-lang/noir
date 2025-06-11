use crate::fuzz_lib::fuzz_target_lib::{FieldRepresentation, WitnessValue};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait ArgumentsMutatorStrategy<'a> {
    fn mutate(&mut self, value: WitnessValue) -> WitnessValue;
    fn new(rng: &'a mut StdRng) -> Self;
}

struct RandomMutation<'a> {
    rng: &'a mut StdRng,
}
impl<'a> ArgumentsMutatorStrategy<'a> for RandomMutation<'a> {
    fn mutate(&mut self, _value: WitnessValue) -> WitnessValue {
        let mut bytes = [0u8; 17];
        self.rng.fill(&mut bytes);
        return Unstructured::new(&bytes).arbitrary().unwrap();
    }

    fn new(rng: &'a mut StdRng) -> Self {
        Self { rng }
    }
}

struct MaxValueMutation<'a> {
    rng: &'a mut StdRng,
}
impl<'a> ArgumentsMutatorStrategy<'a> for MaxValueMutation<'a> {
    fn mutate(&mut self, value: WitnessValue) -> WitnessValue {
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

    fn new(rng: &'a mut StdRng) -> Self {
        Self { rng }
    }
}

struct MinValueMutation<'a> {
    rng: &'a mut StdRng,
}
impl<'a> ArgumentsMutatorStrategy<'a> for MinValueMutation<'a> {
    fn mutate(&mut self, value: WitnessValue) -> WitnessValue {
        match value {
            WitnessValue::Field(_) => WitnessValue::Field(FieldRepresentation { high: 0, low: 0 }),
            WitnessValue::U64(_) => WitnessValue::U64(0),
            WitnessValue::Boolean(_) => WitnessValue::Boolean(false),
            WitnessValue::I64(_) => WitnessValue::I64(1 << 63),
            WitnessValue::I32(_) => WitnessValue::I32(1 << 31),
        }
    }

    fn new(rng: &'a mut StdRng) -> Self {
        Self { rng }
    }
}

struct DefaultMutationStrategy<'a> {
    rng: &'a mut StdRng,
}
impl<'a> ArgumentsMutatorStrategy<'a> for DefaultMutationStrategy<'a> {
    fn mutate(&mut self, value: WitnessValue) -> WitnessValue {
        value
    }

    fn new(rng: &'a mut StdRng) -> Self {
        Self { rng }
    }
}

pub(crate) fn witness_mutator(
    witness_value: Vec<WitnessValue>,
    rng: &mut StdRng,
) -> Vec<WitnessValue> {
    let mut witness_values: Vec<WitnessValue> = Vec::new();
    for witness in witness_value {
        let witness = if rng.gen_bool(0.5) {
            RandomMutation::new(rng).mutate(witness)
        } else if rng.gen_bool(0.3) {
            MaxValueMutation::new(rng).mutate(witness)
        } else if rng.gen_bool(0.2) {
            MinValueMutation::new(rng).mutate(witness)
        } else {
            DefaultMutationStrategy::new(rng).mutate(witness)
        };
        witness_values.push(witness);
    }
    witness_values
}
