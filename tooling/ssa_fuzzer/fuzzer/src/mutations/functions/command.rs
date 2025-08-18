//! This file contains mechanisms for deterministically mutating a given [FuzzerFunctionCommand](crate::fuzz_lib::function_context::FuzzerFunctionCommand) value

use crate::fuzz_lib::function_context::FuzzerFunctionCommand;
use crate::mutations::{
    basic_types::usize::mutate_usize,
    configuration::{
        BASIC_INSERT_CYCLE_MUTATION_CONFIGURATION,
        BASIC_INSERT_FUNCTION_CALL_MUTATION_CONFIGURATION,
        BASIC_INSERT_JMP_IF_BLOCK_MUTATION_CONFIGURATION, BASIC_USIZE_MUTATION_CONFIGURATION,
        InsertCycleMutationOptions, InsertFunctionCallMutationOptions,
        InsertJmpIfBlockMutationOptions,
    },
};
use rand::Rng;
use rand::rngs::StdRng;

pub(crate) fn mutate_fuzzer_function_command(
    command: &mut FuzzerFunctionCommand,
    rng: &mut StdRng,
) {
    match command {
        FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx } => {
            mutate_usize(instruction_block_idx, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
        }
        FuzzerFunctionCommand::InsertJmpIfBlock { block_then_idx, block_else_idx } => {
            match BASIC_INSERT_JMP_IF_BLOCK_MUTATION_CONFIGURATION.select(rng) {
                InsertJmpIfBlockMutationOptions::BlockThenIdx => {
                    mutate_usize(block_then_idx, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                }
                InsertJmpIfBlockMutationOptions::BlockElseIdx => {
                    mutate_usize(block_else_idx, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                }
            }
        }
        FuzzerFunctionCommand::InsertJmpBlock { block_idx } => {
            mutate_usize(block_idx, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
        }
        FuzzerFunctionCommand::SwitchToNextBlock => {}
        FuzzerFunctionCommand::InsertCycle { block_body_idx, start_iter, end_iter } => {
            match BASIC_INSERT_CYCLE_MUTATION_CONFIGURATION.select(rng) {
                InsertCycleMutationOptions::BlockBodyIdx => {
                    mutate_usize(block_body_idx, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                }
                InsertCycleMutationOptions::StartIter => {
                    *start_iter = rng.gen_range(u8::MIN..=u8::MAX);
                }
                InsertCycleMutationOptions::EndIter => {
                    *end_iter = rng.gen_range(u8::MIN..=u8::MAX);
                }
            }
        }
        FuzzerFunctionCommand::InsertFunctionCall { function_idx, args } => {
            match BASIC_INSERT_FUNCTION_CALL_MUTATION_CONFIGURATION.select(rng) {
                InsertFunctionCallMutationOptions::FunctionIdx => {
                    mutate_usize(function_idx, rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                }
                InsertFunctionCallMutationOptions::Args => {
                    let idx = rng.gen_range(0..args.len());
                    mutate_usize(&mut args[idx], rng, BASIC_USIZE_MUTATION_CONFIGURATION);
                }
            }
        }
    }
}
