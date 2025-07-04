mod commands_mutator;
mod function;

use crate::fuzz_lib::function_context::FunctionData;
use crate::mutations::configuration::{
    BASIC_FUNCTION_VEC_MUTATION_CONFIGURATION, FunctionVecMutationOptions,
};
use crate::mutations::functions::function::mutate_function;
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait MutateFunctionVec {
    fn mutate(rng: &mut StdRng, funcs: &mut Vec<FunctionData>);
}
struct RandomMutation;
impl MutateFunctionVec for RandomMutation {
    fn mutate(rng: &mut StdRng, functions: &mut Vec<FunctionData>) {
        let mut bytes = [0u8; 128];
        rng.fill(&mut bytes);
        *functions = Unstructured::new(&bytes).arbitrary().unwrap();
    }
}

struct RemoveFunctionMutation;
impl MutateFunctionVec for RemoveFunctionMutation {
    fn mutate(rng: &mut StdRng, functions: &mut Vec<FunctionData>) {
        if !functions.is_empty() {
            functions.remove(rng.gen_range(0..functions.len()));
        }
    }
}

/// Replace randomly chosen function with randomly generated function
struct ReplaceFunctionMutation;
impl MutateFunctionVec for ReplaceFunctionMutation {
    fn mutate(rng: &mut StdRng, functions: &mut Vec<FunctionData>) {
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        let command = Unstructured::new(&bytes).arbitrary().unwrap();
        if !functions.is_empty() {
            let command_idx = rng.gen_range(0..functions.len());
            functions[command_idx] = command;
        }
    }
}

struct MutateFunction;
impl MutateFunctionVec for MutateFunction {
    fn mutate(rng: &mut StdRng, functions: &mut Vec<FunctionData>) {
        if functions.is_empty() {
            return;
        }
        let function_idx = rng.gen_range(0..functions.len());
        mutate_function(&mut functions[function_idx], rng);
    }
}

/// Insert randomly generated function
struct FunctionInsertionMutation;
impl MutateFunctionVec for FunctionInsertionMutation {
    fn mutate(rng: &mut StdRng, functions: &mut Vec<FunctionData>) {
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        let function = Unstructured::new(&bytes).arbitrary().unwrap();
        if !functions.is_empty() {
            let function_idx = rng.gen_range(0..functions.len());
            functions[function_idx] = function;
        }
    }
}

pub(crate) fn mutate(vec_fuzzer_command: &mut Vec<FunctionData>, rng: &mut StdRng) {
    match BASIC_FUNCTION_VEC_MUTATION_CONFIGURATION.select(rng) {
        FunctionVecMutationOptions::Random => RandomMutation::mutate(rng, vec_fuzzer_command),
        FunctionVecMutationOptions::Remove => {
            RemoveFunctionMutation::mutate(rng, vec_fuzzer_command)
        }
        FunctionVecMutationOptions::Insertion => {
            FunctionInsertionMutation::mutate(rng, vec_fuzzer_command)
        }
        FunctionVecMutationOptions::Replace => {
            ReplaceFunctionMutation::mutate(rng, vec_fuzzer_command)
        }
        FunctionVecMutationOptions::MutateFunc => {
            MutateFunction::mutate(rng, vec_fuzzer_command);
        }
    }
}
