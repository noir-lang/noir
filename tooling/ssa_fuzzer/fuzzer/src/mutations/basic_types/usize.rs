//! This file contains mechanisms for deterministically mutating a given usize
//! Types of mutations applied:
//! 1. Random (randomly select a new usize)
//! 2. Increment by 1
//! 3. Decrement by 1
//! 4. Add a random value
//! 5. Subtract a random value

use crate::mutations::configuration::{UsizeMutationConfig, UsizeMutationOptions};
use rand::{Rng, rngs::StdRng};

trait MutateUsize {
    fn mutate(rng: &mut StdRng, value: &mut usize);
}

struct RandomMutation;
impl MutateUsize for RandomMutation {
    fn mutate(rng: &mut StdRng, value: &mut usize) {
        *value = rng.gen_range(0..usize::MAX);
    }
}

struct IncrementMutation;
impl MutateUsize for IncrementMutation {
    fn mutate(_rng: &mut StdRng, value: &mut usize) {
        *value = value.saturating_add(1);
    }
}

struct DecrementMutation;
impl MutateUsize for DecrementMutation {
    fn mutate(_rng: &mut StdRng, value: &mut usize) {
        *value = value.saturating_sub(1);
    }
}

struct AddRandomMutation;
impl MutateUsize for AddRandomMutation {
    fn mutate(rng: &mut StdRng, value: &mut usize) {
        *value = value.saturating_add(rng.gen_range(0..usize::MAX));
    }
}

struct SubtractRandomMutation;
impl MutateUsize for SubtractRandomMutation {
    fn mutate(rng: &mut StdRng, value: &mut usize) {
        *value = value.saturating_sub(rng.gen_range(0..usize::MAX));
    }
}

pub(crate) fn mutate_usize(usize: &mut usize, rng: &mut StdRng, config: UsizeMutationConfig) {
    match config.select(rng) {
        UsizeMutationOptions::Random => RandomMutation::mutate(rng, usize),
        UsizeMutationOptions::Increment => IncrementMutation::mutate(rng, usize),
        UsizeMutationOptions::Decrement => DecrementMutation::mutate(rng, usize),
        UsizeMutationOptions::AddRandom => AddRandomMutation::mutate(rng, usize),
        UsizeMutationOptions::SubtractRandom => SubtractRandomMutation::mutate(rng, usize),
    }
}
