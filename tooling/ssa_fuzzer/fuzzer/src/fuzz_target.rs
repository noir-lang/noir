#![no_main]

mod fuzz_lib;
use fuzz_lib::fuzz_target_lib::{FuzzerData, fuzz_target};
use fuzz_lib::options::{FuzzerCommandOptions, FuzzerOptions, InstructionOptions};
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
    let instruction_options = InstructionOptions {
        cast_enabled: false,
        xor_enabled: true,
        and_enabled: true,
        or_enabled: true,
        not_enabled: true,
        add_enabled: true,
        sub_enabled: true,
        mul_enabled: true,
        mod_enabled: false,
        div_enabled: true,
        shl_enabled: false,
        shr_enabled: false,
        eq_enabled: true,
        lt_enabled: false,
        load_enabled: true,
        store_enabled: true,
        alloc_enabled: true,
    };
    let fuzzer_command_options = FuzzerCommandOptions {
        merge_instruction_blocks_enabled: true,
        jmp_if_enabled: true,
        jmp_block_enabled: true,
        switch_to_next_block_enabled: true,
    };
    let options = FuzzerOptions {
        idempotent_morphing_enabled: false,
        constant_execution_enabled: false,
        compile_options,
        max_ssa_blocks_num: 30,
        max_instructions_num: 500,
        instruction_options,
        fuzzer_command_options,
    };
    fuzz_target(data, options);
});
