#![no_main]

mod fuzz_lib;
mod mutations;

use bincode::{deserialize, serialize};
use fuzz_lib::function_context::FunctionData;
use fuzz_lib::fuzz_target_lib::fuzz_target;
use fuzz_lib::options::{FuzzerOptions, InstructionOptions};
use mutations::mutate;
use noirc_driver::CompileOptions;
use rand::{SeedableRng, rngs::StdRng};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let _ = env_logger::try_init();
    let mut compile_options = CompileOptions::default();
    if let Ok(triage_value) = std::env::var("TRIAGE") {
        match triage_value.as_str() {
            "FULL" => compile_options.show_ssa = true,
            "FINAL" => {
                compile_options.show_ssa_pass =
                    vec!["After Dead Instruction Elimination - ACIR".to_string()]
            }
            "FIRST_AND_FINAL" => {
                compile_options.show_ssa_pass = vec![
                    "After Removing Unreachable Functions (1)".to_string(),
                    "After Dead Instruction Elimination - ACIR".to_string(),
                ]
            }
            _ => (),
        }
    }

    // Disable some instructions with bugs that are not fixed yet
    let instruction_options = InstructionOptions {
        shl_enabled: false,
        shr_enabled: false,
        ..InstructionOptions::default()
    };
    let options =
        FuzzerOptions { compile_options, instruction_options, ..FuzzerOptions::default() };
    let data: Vec<FunctionData> = deserialize(data).unwrap_or(vec![FunctionData::default()]);
    fuzz_target(data, options);
});

libfuzzer_sys::fuzz_mutator!(|data: &mut [u8], _size: usize, max_size: usize, seed: u32| {
    let mut rng = StdRng::seed_from_u64(seed as u64);
    let mut new_fuzzer_data: Vec<FunctionData> = deserialize(data).unwrap_or_default();
    mutate(&mut new_fuzzer_data, &mut rng);
    let new_bytes = serialize(&new_fuzzer_data).unwrap();
    if new_bytes.len() > max_size {
        return 0;
    }
    data[..new_bytes.len()].copy_from_slice(&new_bytes);
    new_bytes.len()
});
