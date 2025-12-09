//! This file contains mechanisms for deterministically mutating a given bool

use crate::mutations::configuration::{
    BoolMutationConfig, BoolMutationOptions, GenerateBool, GenerateBoolConfig,
};
use rand::rngs::StdRng;

pub(crate) fn generate_random_bool(rng: &mut StdRng, config: GenerateBoolConfig) -> bool {
    match config.select(rng) {
        GenerateBool::True => true,
        GenerateBool::False => false,
    }
}

pub(crate) fn mutate_bool(bool: &mut bool, rng: &mut StdRng, config: BoolMutationConfig) {
    match config.select(rng) {
        BoolMutationOptions::True => *bool = true,
        BoolMutationOptions::False => *bool = false,
    }
}
