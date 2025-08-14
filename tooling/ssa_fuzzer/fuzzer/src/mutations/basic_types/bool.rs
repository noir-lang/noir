//! This file contains mechanisms for deterministically mutating a given bool

use crate::mutations::configuration::{BoolMutationConfig, BoolMutationOptions};
use rand::rngs::StdRng;

pub(crate) fn mutate_bool(bool: &mut bool, rng: &mut StdRng, config: BoolMutationConfig) {
    match config.select(rng) {
        BoolMutationOptions::True => *bool = true,
        BoolMutationOptions::False => *bool = false,
    }
}
