//! This file contains mechanisms for deterministically mutating a given vector of [FuzzerCommand](crate::fuzz_lib::base_context::FuzzerCommand) values
//! Types of mutations applied:
//! 1. Random (randomly select a new vector of fuzzer commands)
//! 2. Remove command
//! 3. Add command
//! 4. Replace command with random command

use crate::fuzz_lib::function_context::FuzzerFunctionCommand;
use crate::mutations::configuration::{
    BASIC_FUZZER_COMMAND_MUTATION_CONFIGURATION, FuzzerCommandMutationOptions,
};
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait MutateVecFuzzerCommand {
    fn mutate(rng: &mut StdRng, commands: &mut Vec<FuzzerFunctionCommand>);
}

/// Return new random vector of fuzzer commands
struct RandomMutation;
impl MutateVecFuzzerCommand for RandomMutation {
    fn mutate(rng: &mut StdRng, commands: &mut Vec<FuzzerFunctionCommand>) {
        let mut bytes = [0u8; 128];
        rng.fill(&mut bytes);
        *commands = Unstructured::new(&bytes).arbitrary().unwrap();
    }
}

/// Remove randomly chosen command from the vector
struct RemoveCommandMutation;
impl MutateVecFuzzerCommand for RemoveCommandMutation {
    fn mutate(rng: &mut StdRng, commands: &mut Vec<FuzzerFunctionCommand>) {
        if !commands.is_empty() {
            commands.remove(rng.gen_range(0..commands.len()));
        }
    }
}

/// Add randomly generated command to the vector
struct AddCommandMutation;
impl MutateVecFuzzerCommand for AddCommandMutation {
    fn mutate(rng: &mut StdRng, commands: &mut Vec<FuzzerFunctionCommand>) {
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        let command = Unstructured::new(&bytes).arbitrary().unwrap();
        commands.push(command);
    }
}

/// Replace randomly chosen command with randomly generated command
struct ReplaceCommandMutation;
impl MutateVecFuzzerCommand for ReplaceCommandMutation {
    fn mutate(rng: &mut StdRng, commands: &mut Vec<FuzzerFunctionCommand>) {
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        let command = Unstructured::new(&bytes).arbitrary().unwrap();
        if !commands.is_empty() {
            let command_idx = rng.gen_range(0..commands.len());
            commands[command_idx] = command;
        }
    }
}

pub(crate) fn mutate_vec_fuzzer_command(
    vec_fuzzer_command: &mut Vec<FuzzerFunctionCommand>,
    rng: &mut StdRng,
) {
    match BASIC_FUZZER_COMMAND_MUTATION_CONFIGURATION.select(rng) {
        FuzzerCommandMutationOptions::Random => RandomMutation::mutate(rng, vec_fuzzer_command),
        FuzzerCommandMutationOptions::RemoveCommand => {
            RemoveCommandMutation::mutate(rng, vec_fuzzer_command)
        }
        FuzzerCommandMutationOptions::AddCommand => {
            AddCommandMutation::mutate(rng, vec_fuzzer_command)
        }
        FuzzerCommandMutationOptions::ReplaceCommand => {
            ReplaceCommandMutation::mutate(rng, vec_fuzzer_command)
        }
    }
}
