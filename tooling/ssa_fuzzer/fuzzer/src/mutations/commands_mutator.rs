//! This file contains mechanisms for deterministically mutating a given vector of [FuzzerCommand](crate::fuzz_lib::base_context::FuzzerCommand) values
//! Types of mutations applied:
//! 1. Random (randomly select a new vector of fuzzer commands)
//! 2. Remove command
//! 3. Add command
//! 4. Replace command with random command

use crate::fuzz_lib::base_context::FuzzerCommand;
use crate::mutations::configuration::{
    BASIC_FUZZER_COMMAND_MUTATION_CONFIGURATION, FuzzerCommandMutationOptions,
};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait MutateVecFuzzerCommand {
    fn mutate(&self, rng: &mut StdRng, value: Vec<FuzzerCommand>) -> Vec<FuzzerCommand>;
}

trait MutateVecFuzzerCommandFactory {
    fn new_box() -> Box<dyn MutateVecFuzzerCommand>;
}

/// Return new random vector of fuzzer commands
#[derive(Default)]
struct RandomMutation;
impl MutateVecFuzzerCommand for RandomMutation {
    fn mutate(&self, rng: &mut StdRng, _value: Vec<FuzzerCommand>) -> Vec<FuzzerCommand> {
        let mut bytes = [0u8; 128];
        rng.fill(&mut bytes);
        Unstructured::new(&bytes).arbitrary().unwrap()
    }
}

/// Remove randomly chosen command from the vector
#[derive(Default)]
struct RemoveCommandMutation;
impl MutateVecFuzzerCommand for RemoveCommandMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<FuzzerCommand>) -> Vec<FuzzerCommand> {
        let mut commands = value;
        if !commands.is_empty() {
            commands.remove(rng.gen_range(0..commands.len()));
        }
        commands
    }
}

/// Add randomly generated command to the vector
#[derive(Default)]
struct AddCommandMutation;
impl MutateVecFuzzerCommand for AddCommandMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<FuzzerCommand>) -> Vec<FuzzerCommand> {
        let mut commands = value.clone();
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        let command = Unstructured::new(&bytes).arbitrary().unwrap();
        commands.push(command);
        commands
    }
}

/// Replace randomly chosen command with randomly generated command
#[derive(Default)]
struct ReplaceCommandMutation;
impl MutateVecFuzzerCommand for ReplaceCommandMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<FuzzerCommand>) -> Vec<FuzzerCommand> {
        let mut commands = value;
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        let command = Unstructured::new(&bytes).arbitrary().unwrap();
        if !commands.is_empty() {
            let command_idx = rng.gen_range(0..commands.len());
            commands[command_idx] = command;
        }
        commands
    }
}

impl<T> MutateVecFuzzerCommandFactory for T
where
    T: MutateVecFuzzerCommand + Default + 'static,
{
    fn new_box() -> Box<dyn MutateVecFuzzerCommand> {
        Box::new(T::default())
    }
}

// todo more mutations
fn mutation_factory(rng: &mut StdRng) -> Box<dyn MutateVecFuzzerCommand> {
    match BASIC_FUZZER_COMMAND_MUTATION_CONFIGURATION.select(rng) {
        FuzzerCommandMutationOptions::Random => RandomMutation::new_box(),
        FuzzerCommandMutationOptions::RemoveCommand => RemoveCommandMutation::new_box(),
        FuzzerCommandMutationOptions::AddCommand => AddCommandMutation::new_box(),
        FuzzerCommandMutationOptions::ReplaceCommand => ReplaceCommandMutation::new_box(),
    }
}

pub(crate) fn mutate_vec_fuzzer_command(
    vec_fuzzer_command: Vec<FuzzerCommand>,
    rng: &mut StdRng,
) -> Vec<FuzzerCommand> {
    let mutator = mutation_factory(rng);
    mutator.mutate(rng, vec_fuzzer_command)
}
