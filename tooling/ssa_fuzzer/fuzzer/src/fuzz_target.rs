#![no_main]

mod fuzz_lib;
use fuzz_lib::fuzz_lib::{FuzzerData, fuzz_target};
use fuzz_lib::options::FuzzerOptions;
use noirc_driver::CompileOptions;

libfuzzer_sys::fuzz_target!(|data: FuzzerData| {
    let _ = env_logger::try_init();
    let mut compile_options = CompileOptions::default();
    if let Ok(triage_value) = std::env::var("TRIAGE") {
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
