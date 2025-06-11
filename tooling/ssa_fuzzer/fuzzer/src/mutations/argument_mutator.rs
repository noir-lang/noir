use crate::fuzz_lib::instruction::Argument;
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait ArgumentsMutatorStrategy<'a> {
    fn mutate(&mut self, value: Argument) -> Argument;
    fn new(rng: &'a mut StdRng) -> Self;
}

struct RandomMutation<'a> {
    rng: &'a mut StdRng,
}
impl<'a> ArgumentsMutatorStrategy<'a> for RandomMutation<'a> {
    fn mutate(&mut self, value: Argument) -> Argument {
        let mut bytes = [0u8; 17];
        self.rng.fill(&mut bytes);
        return Unstructured::new(&bytes).arbitrary().unwrap();
    }

    fn new(rng: &'a mut StdRng) -> Self {
        Self { rng }
    }
}

struct IncrementArgumentIndexMutation<'a> {
    rng: &'a mut StdRng,
}
impl<'a> ArgumentsMutatorStrategy<'a> for IncrementArgumentIndexMutation<'a> {
    fn mutate(&mut self, value: Argument) -> Argument {
        Argument { index: value.index + 1, value_type: value.value_type }
    }

    fn new(rng: &'a mut StdRng) -> Self {
        Self { rng }
    }
}

struct DecrementArgumentIndexMutation<'a> {
    rng: &'a mut StdRng,
}
impl<'a> ArgumentsMutatorStrategy<'a> for DecrementArgumentIndexMutation<'a> {
    fn mutate(&mut self, value: Argument) -> Argument {
        Argument { index: value.index - 1, value_type: value.value_type }
    }

    fn new(rng: &'a mut StdRng) -> Self {
        Self { rng }
    }
}

struct ChangeTypeMutation<'a> {
    rng: &'a mut StdRng,
}
impl<'a> ArgumentsMutatorStrategy<'a> for ChangeTypeMutation<'a> {
    fn mutate(&mut self, value: Argument) -> Argument {
        let mut bytes = [0u8; 17];
        self.rng.fill(&mut bytes);
        let value_type = Unstructured::new(&bytes).arbitrary().unwrap();
        Argument { index: value.index, value_type }
    }

    fn new(rng: &'a mut StdRng) -> Self {
        Self { rng }
    }
}

struct DefaultMutationStrategy<'a> {
    rng: &'a mut StdRng,
}
impl<'a> ArgumentsMutatorStrategy<'a> for DefaultMutationStrategy<'a> {
    fn mutate(&mut self, value: Argument) -> Argument {
        value
    }

    fn new(rng: &'a mut StdRng) -> Self {
        Self { rng }
    }
}

pub(crate) fn argument_mutator(argument: Argument, rng: &mut StdRng) -> Argument {
    let argument = if rng.gen_bool(0.5) {
        RandomMutation::new(rng).mutate(argument)
    } else if rng.gen_bool(0.3) {
        IncrementArgumentIndexMutation::new(rng).mutate(argument)
    } else if rng.gen_bool(0.2) {
        DecrementArgumentIndexMutation::new(rng).mutate(argument)
    } else if rng.gen_bool(0.2) {
        ChangeTypeMutation::new(rng).mutate(argument)
    } else {
        DefaultMutationStrategy::new(rng).mutate(argument)
    };
    argument
}
