use crate::fuzz_lib::instruction::Argument;
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait ArgumentsMutator {
    fn mutate(&self, rng: &mut StdRng, value: Argument) -> Argument;
}
trait ArgumentsMutatorFactory {
    fn new() -> Box<dyn ArgumentsMutator>;
}

struct RandomMutation;
impl ArgumentsMutator for RandomMutation {
    fn mutate(&self, rng: &mut StdRng, value: Argument) -> Argument {
        let mut bytes = [0u8; 17];
        rng.fill(&mut bytes);
        return Unstructured::new(&bytes).arbitrary().unwrap();
    }
}
impl ArgumentsMutatorFactory for RandomMutation {
    fn new() -> Box<dyn ArgumentsMutator> {
        Box::new(RandomMutation)
    }
}

struct IncrementArgumentIndexMutation;
impl ArgumentsMutator for IncrementArgumentIndexMutation {
    fn mutate(&self, rng: &mut StdRng, value: Argument) -> Argument {
        Argument { index: value.index + 1, value_type: value.value_type }
    }
}
impl ArgumentsMutatorFactory for IncrementArgumentIndexMutation {
    fn new() -> Box<dyn ArgumentsMutator> {
        Box::new(IncrementArgumentIndexMutation)
    }
}

struct DecrementArgumentIndexMutation;
impl ArgumentsMutator for DecrementArgumentIndexMutation {
    fn mutate(&self, rng: &mut StdRng, value: Argument) -> Argument {
        Argument { index: value.index - 1, value_type: value.value_type }
    }
}
impl ArgumentsMutatorFactory for DecrementArgumentIndexMutation {
    fn new() -> Box<dyn ArgumentsMutator> {
        Box::new(DecrementArgumentIndexMutation)
    }
}

struct ChangeTypeMutation;
impl ArgumentsMutator for ChangeTypeMutation {
    fn mutate(&self, rng: &mut StdRng, value: Argument) -> Argument {
        let mut bytes = [0u8; 17];
        rng.fill(&mut bytes);
        let value_type = Unstructured::new(&bytes).arbitrary().unwrap();
        Argument { index: value.index, value_type }
    }
}
impl ArgumentsMutatorFactory for ChangeTypeMutation {
    fn new() -> Box<dyn ArgumentsMutator> {
        Box::new(ChangeTypeMutation)
    }
}

struct DefaultMutation;
impl ArgumentsMutator for DefaultMutation {
    fn mutate(&self, rng: &mut StdRng, value: Argument) -> Argument {
        value
    }
}
impl ArgumentsMutatorFactory for DefaultMutation {
    fn new() -> Box<dyn ArgumentsMutator> {
        Box::new(DefaultMutation)
    }
}

fn mutation_factory(rng: &mut StdRng) -> Box<dyn ArgumentsMutator> {
    let mutator = if rng.gen_bool(0.9) {
        IncrementArgumentIndexMutation::new()
    } else if rng.gen_bool(0.9) {
        DecrementArgumentIndexMutation::new()
    } else if rng.gen_bool(0.5) {
        ChangeTypeMutation::new()
    } else if rng.gen_bool(0.1) {
        RandomMutation::new()
    } else {
        DefaultMutation::new()
    };
    mutator
}

pub(crate) fn argument_mutator(argument: Argument, rng: &mut StdRng) -> Argument {
    let mutator = mutation_factory(rng);
    mutator.mutate(rng, argument)
}
