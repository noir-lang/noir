#![no_main]
mod base_context;
mod block_context;
mod fuzz_lib;
mod fuzzer;
mod instruction;
mod options;
use fuzz_lib::{FuzzerData, fuzz_target};
use noirc_driver::CompileOptions;
use options::FuzzerOptions;

libfuzzer_sys::fuzz_target!(|data: FuzzerData| {
    let _ = env_logger::try_init();
    let mut compile_options = CompileOptions::default();
    // Check if we're in triage mode by reading the TRIAGE environment variable
    if let Ok(triage_value) = std::env::var("TRIAGE") {
        log::debug!("Running in triage mode with TRIAGE={}", triage_value);

        match triage_value.as_str() {
            "FULL" => compile_options.show_ssa = true,
            "FINAL" => {
                compile_options.show_ssa_pass = Some("Dead Instruction Elimination (3)".to_string())
            }
            _ => (),
        }
    }
    let options = FuzzerOptions {
        idempotent_morphing_enabled: false,
        constant_execution_enabled: false,
        compile_options,
        max_jumps_num: 30,
        max_instructions_num: 500,
    };
    fuzz_target(data, options);
});
