mod commands_mutator;
mod function;

use crate::fuzz_lib::function_context::FunctionData;
use crate::mutations::configuration::{
    BASIC_FUNCTION_VEC_MUTATION_CONFIGURATION, FunctionVecMutationOptions,
    SIZE_OF_SMALL_ARBITRARY_BUFFER,
};
use crate::mutations::functions::function::mutate_function;
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait MutateFunctionVec {
    fn mutate(rng: &mut StdRng, functions: &mut Vec<FunctionData>);
}

/// Remove random chosen function
struct RemoveFunctionMutation;
impl MutateFunctionVec for RemoveFunctionMutation {
    fn mutate(rng: &mut StdRng, functions: &mut Vec<FunctionData>) {
        if !functions.is_empty() {
            functions.remove(rng.gen_range(0..functions.len()));
        }
    }
}

/// Mutate random chosen function
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

/// Copy random chosen function, push it to the end of the vector
struct CopyFunctionMutation;
impl MutateFunctionVec for CopyFunctionMutation {
    fn mutate(rng: &mut StdRng, functions: &mut Vec<FunctionData>) {
        if functions.is_empty() {
            return;
        }
        let function_idx = rng.gen_range(0..functions.len());
        let function = functions[function_idx].clone();
        functions.push(function);
    }
}

/// Insert empty function
struct InsertEmptyFunctionMutation;
impl MutateFunctionVec for InsertEmptyFunctionMutation {
    fn mutate(_rng: &mut StdRng, functions: &mut Vec<FunctionData>) {
        functions.push(FunctionData::default());
    }
}

/// Insert randomly generated function
struct FunctionInsertionMutation;
impl MutateFunctionVec for FunctionInsertionMutation {
    fn mutate(rng: &mut StdRng, functions: &mut Vec<FunctionData>) {
        let mut bytes = [0u8; SIZE_OF_SMALL_ARBITRARY_BUFFER];
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
        FunctionVecMutationOptions::CopyFunction => {
            CopyFunctionMutation::mutate(rng, vec_fuzzer_command)
        }
        FunctionVecMutationOptions::Remove => {
            RemoveFunctionMutation::mutate(rng, vec_fuzzer_command)
        }
        FunctionVecMutationOptions::Insertion => {
            FunctionInsertionMutation::mutate(rng, vec_fuzzer_command)
        }
        FunctionVecMutationOptions::InsertEmpty => {
            InsertEmptyFunctionMutation::mutate(rng, vec_fuzzer_command)
        }
        FunctionVecMutationOptions::MutateFunction => {
            MutateFunction::mutate(rng, vec_fuzzer_command);
        }
    }
}
