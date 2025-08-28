//! This file contains mechanisms for deterministically mutating a given vector of [FuzzerCommand](crate::fuzz_lib::base_context::FuzzerCommand) values

use crate::fuzz_lib::{NUMBER_OF_VARIABLES_INITIAL, function_context::FuzzerFunctionCommand};
use crate::mutations::{
    basic_types::vec::mutate_vec,
    configuration::{
        BASIC_GENERATE_COMMAND_CONFIGURATION, BASIC_VEC_MUTATION_CONFIGURATION, GenerateCommand,
    },
    functions::command::mutate_fuzzer_function_command,
};
use rand::rngs::StdRng;

pub(crate) fn generate_random_fuzzer_function_command(rng: &mut StdRng) -> FuzzerFunctionCommand {
    match BASIC_GENERATE_COMMAND_CONFIGURATION.select(rng) {
        GenerateCommand::InsertJmpIfBlock => {
            FuzzerFunctionCommand::InsertJmpIfBlock { block_then_idx: 0, block_else_idx: 0 }
        }
        GenerateCommand::InsertJmpBlock => FuzzerFunctionCommand::InsertJmpBlock { block_idx: 0 },
        GenerateCommand::InsertCycle => {
            FuzzerFunctionCommand::InsertCycle { block_body_idx: 0, start_iter: 0, end_iter: 0 }
        }
        GenerateCommand::InsertFunctionCall => FuzzerFunctionCommand::InsertFunctionCall {
            function_idx: 0,
            args: [0; NUMBER_OF_VARIABLES_INITIAL as usize],
        },
        GenerateCommand::InsertSimpleInstructionBlock => {
            FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx: 0 }
        }
        GenerateCommand::SwitchToNextBlock => FuzzerFunctionCommand::SwitchToNextBlock,
    }
}

pub(crate) fn mutate_vec_fuzzer_command(
    vec_fuzzer_command: &mut Vec<FuzzerFunctionCommand>,
    rng: &mut StdRng,
) {
    mutate_vec(
        vec_fuzzer_command,
        rng,
        mutate_fuzzer_function_command,
        generate_random_fuzzer_function_command,
        BASIC_VEC_MUTATION_CONFIGURATION,
    );
}
