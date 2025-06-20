//! This file contains mechanisms for deterministically mutating a given [Argument](crate::fuzz_lib::instruction::Argument) value
//! Types of mutations applied:
//! 1. Random (randomly select a new argument value)
//! 2. Increment index
//! 3. Decrement index
//! 4. Change type

use crate::fuzz_lib::instruction::Argument;
use crate::mutations::configuration::{
    ArgumentMutationOptions, BASIC_ARGUMENT_MUTATION_CONFIGURATION,
};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait ArgumentsMutator {
    fn mutate(&self, rng: &mut StdRng, value: Argument) -> Argument;
}
trait ArgumentsMutatorFactory {
    fn new() -> Box<dyn ArgumentsMutator>;
}

/// Return new random argument
struct RandomMutation;
impl ArgumentsMutator for RandomMutation {
    fn mutate(&self, rng: &mut StdRng, _value: Argument) -> Argument {
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

/// Increment index of the argument
struct IncrementArgumentIndexMutation;
impl ArgumentsMutator for IncrementArgumentIndexMutation {
    fn mutate(&self, _rng: &mut StdRng, value: Argument) -> Argument {
        Argument { index: value.index + 1, value_type: value.value_type }
    }
}
impl ArgumentsMutatorFactory for IncrementArgumentIndexMutation {
    fn new() -> Box<dyn ArgumentsMutator> {
        Box::new(IncrementArgumentIndexMutation)
    }
}

/// Decrement index of the argument
struct DecrementArgumentIndexMutation;
impl ArgumentsMutator for DecrementArgumentIndexMutation {
    fn mutate(&self, _rng: &mut StdRng, value: Argument) -> Argument {
        Argument { index: value.index.saturating_sub(1), value_type: value.value_type }
    }
}
impl ArgumentsMutatorFactory for DecrementArgumentIndexMutation {
    fn new() -> Box<dyn ArgumentsMutator> {
        Box::new(DecrementArgumentIndexMutation)
    }
}

/// Change type of the argument
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

fn mutation_factory(rng: &mut StdRng) -> Box<dyn ArgumentsMutator> {
    let mutator = match BASIC_ARGUMENT_MUTATION_CONFIGURATION.select(rng) {
        ArgumentMutationOptions::Random => RandomMutation::new(),
        ArgumentMutationOptions::IncrementIndex => IncrementArgumentIndexMutation::new(),
        ArgumentMutationOptions::DecrementIndex => DecrementArgumentIndexMutation::new(),
        ArgumentMutationOptions::ChangeType => ChangeTypeMutation::new(),
    };
    mutator
}

pub(crate) fn argument_mutator(argument: Argument, rng: &mut StdRng) -> Argument {
    let mutator = mutation_factory(rng);
    mutator.mutate(rng, argument)
}
