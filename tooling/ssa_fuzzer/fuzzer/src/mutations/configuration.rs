//! This file contains configurations for selecting particular behaviors during mutations
use rand::{Rng, rngs::StdRng};

pub(crate) const MAX_NUMBER_OF_MUTATIONS: usize = 25;

pub(crate) struct WeightedSelectionConfig<T, const N: usize> {
    pub(crate) options_with_weights: [(T, usize); N],
    pub(crate) total_weight: usize,
}

impl<T: Copy, const N: usize> WeightedSelectionConfig<T, N> {
    pub(crate) const fn new(options_with_weights: [(T, usize); N]) -> Self {
        let mut total_weight = 0;
        let mut i = 0;
        while i < options_with_weights.len() {
            total_weight += options_with_weights[i].1;
            i += 1;
        }

        Self { options_with_weights, total_weight }
    }

    pub(crate) fn select(&self, rng: &mut StdRng) -> T {
        let mut selector = rng.gen_range(0..self.total_weight);
        for (option, weight) in &self.options_with_weights {
            if selector < *weight {
                return *option;
            }
            selector -= weight;
        }
        unreachable!("Should have returned by now")
    }
}

/// Mutations config for single mutation
#[derive(Copy, Clone, Debug)]
pub(crate) enum MutationOptions {
    InstructionBlocks,
    FuzzerCommands,
    Witnesses,
}

pub(crate) type MutationConfig = WeightedSelectionConfig<MutationOptions, 3>;
pub(crate) const BASIC_MUTATION_CONFIGURATION: MutationConfig = MutationConfig::new([
    (MutationOptions::InstructionBlocks, 1),
    (MutationOptions::FuzzerCommands, 1),
    (MutationOptions::Witnesses, 1),
]);

/// Mutations of witness values
#[derive(Copy, Clone, Debug)]
pub(crate) enum WitnessMutationOptions {
    Random,
    MaxValue,
    MinValue,
}

pub(crate) type WitnessMutationConfig = WeightedSelectionConfig<WitnessMutationOptions, 3>;
pub(crate) const BASIC_WITNESS_MUTATION_CONFIGURATION: WitnessMutationConfig =
    WitnessMutationConfig::new([
        (WitnessMutationOptions::Random, 1),
        (WitnessMutationOptions::MaxValue, 3),
        (WitnessMutationOptions::MinValue, 2),
    ]);

/// Mutations of fuzzer commands
#[derive(Copy, Clone, Debug)]
pub(crate) enum FuzzerCommandMutationOptions {
    Random,
    RemoveCommand,
    AddCommand,
    ReplaceCommand,
}

pub(crate) type FuzzerCommandMutationConfig =
    WeightedSelectionConfig<FuzzerCommandMutationOptions, 4>;
pub(crate) const BASIC_FUZZER_COMMAND_MUTATION_CONFIGURATION: FuzzerCommandMutationConfig =
    FuzzerCommandMutationConfig::new([
        (FuzzerCommandMutationOptions::Random, 1),
        (FuzzerCommandMutationOptions::RemoveCommand, 2),
        (FuzzerCommandMutationOptions::AddCommand, 4),
        (FuzzerCommandMutationOptions::ReplaceCommand, 3),
    ]);

/// Mutations of vector of instruction blocks
#[derive(Copy, Clone, Debug)]
pub(crate) enum VectorOfInstructionBlocksMutationOptions {
    Random,
    InstructionBlockDeletion,
    InstructionBlockInsertion,
    InstructionBlockMutation,
}

pub(crate) type VectorOfInstructionBlocksMutationConfig =
    WeightedSelectionConfig<VectorOfInstructionBlocksMutationOptions, 4>;
pub(crate) const BASIC_VECTOR_OF_INSTRUCTION_BLOCKS_MUTATION_CONFIGURATION:
    VectorOfInstructionBlocksMutationConfig = VectorOfInstructionBlocksMutationConfig::new([
    (VectorOfInstructionBlocksMutationOptions::Random, 1),
    (VectorOfInstructionBlocksMutationOptions::InstructionBlockDeletion, 15),
    (VectorOfInstructionBlocksMutationOptions::InstructionBlockInsertion, 15),
    (VectorOfInstructionBlocksMutationOptions::InstructionBlockMutation, 55),
]);

/// Mutations of single instruction block
#[derive(Copy, Clone, Debug)]
pub(crate) enum InstructionBlockMutationOptions {
    Random,
    InstructionDeletion,
    InstructionInsertion,
    InstructionMutation,
}

pub(crate) type InstructionBlockMutationConfig =
    WeightedSelectionConfig<InstructionBlockMutationOptions, 4>;
pub(crate) const BASIC_INSTRUCTION_BLOCK_MUTATION_CONFIGURATION: InstructionBlockMutationConfig =
    InstructionBlockMutationConfig::new([
        (InstructionBlockMutationOptions::Random, 1),
        (InstructionBlockMutationOptions::InstructionDeletion, 15),
        (InstructionBlockMutationOptions::InstructionInsertion, 15),
        (InstructionBlockMutationOptions::InstructionMutation, 55),
    ]);

/// Mutations of instructions
#[derive(Copy, Clone, Debug)]
pub(crate) enum InstructionMutationOptions {
    Random,
    ArgumentMutation,
}

pub(crate) type InstructionMutationConfig = WeightedSelectionConfig<InstructionMutationOptions, 2>;
pub(crate) const BASIC_INSTRUCTION_MUTATION_CONFIGURATION: InstructionMutationConfig =
    InstructionMutationConfig::new([
        (InstructionMutationOptions::Random, 1),
        (InstructionMutationOptions::ArgumentMutation, 4),
    ]);

/// Mutations of arguments of instructions
#[derive(Copy, Clone, Debug)]
pub(crate) enum ArgumentMutationOptions {
    Random,
    IncrementIndex,
    DecrementIndex,
    ChangeType,
}

pub(crate) type ArgumentMutationConfig = WeightedSelectionConfig<ArgumentMutationOptions, 4>;
pub(crate) const BASIC_ARGUMENT_MUTATION_CONFIGURATION: ArgumentMutationConfig =
    ArgumentMutationConfig::new([
        (ArgumentMutationOptions::Random, 1),
        (ArgumentMutationOptions::IncrementIndex, 3),
        (ArgumentMutationOptions::DecrementIndex, 3),
        (ArgumentMutationOptions::ChangeType, 2),
    ]);
