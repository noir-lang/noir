mod basic_types;
mod configuration;
mod functions;
mod initial_witness;
mod instructions;

use crate::fuzz_lib::{
    function_context::FunctionData,
    fuzzer::FuzzerData,
    instruction::{Instruction, InstructionBlock},
};
use crate::mutations::configuration::{
    BASIC_FUZZER_DATA_MUTATION_CONFIGURATION, FuzzerDataMutationOptions, MAX_NUMBER_OF_MUTATIONS,
};
use noir_ssa_fuzzer::typed_value::Type;
use rand::{Rng, rngs::StdRng};

fn sanitize_type(type_: &mut Type) {
    if type_.is_nested_vector() {
        *type_ = Type::default();
    }
}

fn sanitize_function(function: &mut FunctionData) {
    for input_type in &mut function.input_types {
        sanitize_type(input_type);
    }
    sanitize_type(&mut function.return_type);
}

fn sanitize_instruction(instruction: &mut Instruction) {
    match instruction {
        Instruction::AddToMemory { lhs } | Instruction::LoadFromMemory { memory_addr: lhs } => {
            sanitize_type(&mut lhs.value_type);
        }
        Instruction::SetToMemory { value, .. } => {
            sanitize_type(&mut value.value_type);
        }
        Instruction::CreateArray { element_type, .. } => {
            sanitize_type(element_type);
        }
        _ => {}
    }
}

fn sanitize_instruction_block(instruction_block: &mut InstructionBlock) {
    for instruction in &mut instruction_block.instructions {
        sanitize_instruction(instruction);
    }
}

fn sanitize_fuzzer_data(data: &mut FuzzerData) {
    for function in &mut data.functions {
        sanitize_function(function);
    }
    for instruction_block in &mut data.instruction_blocks {
        sanitize_instruction_block(instruction_block);
    }
}

pub(crate) fn mutate(data: &mut FuzzerData, rng: &mut StdRng) {
    let number_of_mutations = rng.random_range(1..MAX_NUMBER_OF_MUTATIONS);
    for _ in 0..number_of_mutations {
        match BASIC_FUZZER_DATA_MUTATION_CONFIGURATION.select(rng) {
            FuzzerDataMutationOptions::Functions => {
                functions::mutate(&mut data.functions, rng);
            }
            FuzzerDataMutationOptions::InstructionBlocks => {
                instructions::mutate(&mut data.instruction_blocks, rng);
            }
            FuzzerDataMutationOptions::Witnesses => {
                initial_witness::mutate(&mut data.initial_witness, rng);
            }
        }
    }
    sanitize_fuzzer_data(data);
}
