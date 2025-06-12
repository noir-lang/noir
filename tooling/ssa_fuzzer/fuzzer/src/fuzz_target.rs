#![no_main]

mod fuzz_lib;
mod mutations;

use fuzz_lib::fuzz_target_lib::{FuzzerData, fuzz_target};
use fuzz_lib::options::{FuzzerCommandOptions, FuzzerOptions, InstructionOptions};
use mutations::mutate;
use noirc_driver::CompileOptions;

libfuzzer_sys::fuzz_target!(|data: FuzzerData| {
    let _ = env_logger::try_init();
    let mut compile_options = CompileOptions::default();
    if let Ok(triage_value) = std::env::var("TRIAGE") {
        match triage_value.as_str() {
            "FULL" => compile_options.show_ssa = true,
            "FINAL" => {
                compile_options.show_ssa_pass = vec!["Dead Instruction Elimination (3)".to_string()]
            }
            "FIRST_AND_FINAL" => {
                compile_options.show_ssa_pass = vec![
                    "After Removing Unreachable Functions (1)".to_string(),
                    "Dead Instruction Elimination (3)".to_string(),
                ]
            }
            _ => (),
        }
    }

    // Disable some instructions with bugs that are not fixed yet
    let instruction_options = InstructionOptions {
        cast_enabled: true,
        lt_enabled: true,
        shl_enabled: false,
        shr_enabled: false,
        mod_enabled: true,
        ..InstructionOptions::default()
    };
    let options = FuzzerOptions {
        constrain_idempotent_morphing_enabled: false,
        constant_execution_enabled: false,
        compile_options,
        max_ssa_blocks_num: 30, // it takes too long to run program with more blocks
        max_instructions_num: 1500, // it takes too long to run program with more instructions
        max_iterations_num: 10000,
        instruction_options,
        fuzzer_command_options: FuzzerCommandOptions::default(),
    };
    fuzz_target(data, options);
});
