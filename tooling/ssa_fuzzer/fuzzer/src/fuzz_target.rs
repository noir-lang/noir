#![no_main]

mod fuzz_lib;
mod mutations;
mod utils;

use bincode::serde::{borrow_decode_from_slice, encode_to_vec};
use fuzz_lib::fuzz_target_lib::fuzz_target;
use fuzz_lib::fuzzer::FuzzerData;
use fuzz_lib::options::{FuzzerOptions, InstructionOptions};
use libfuzzer_sys::Corpus;
use mutations::mutate;
use noirc_driver::CompileOptions;
use rand::{SeedableRng, rngs::StdRng};
use sha1::{Digest, Sha1};
use utils::{push_fuzzer_output_to_redis_queue, redis};

use crate::fuzz_lib::options::FuzzerMode;

const MAX_EXECUTION_TIME_TO_KEEP_IN_CORPUS: u64 = 3;

libfuzzer_sys::fuzz_target!(|data: &[u8]| -> Corpus {
    let _ = env_logger::try_init();

    let mut compile_options = CompileOptions::default();
    if let Ok(triage_value) = std::env::var("TRIAGE") {
        match triage_value.as_str() {
            "FULL" => compile_options.show_ssa = true,
            "FINAL" => {
                compile_options.show_ssa_pass =
                    vec!["After Dead Instruction Elimination - ACIR".to_string()];
            }
            "FIRST_AND_FINAL" => {
                compile_options.show_ssa_pass = vec![
                    "After Removing Unreachable Functions (1)".to_string(),
                    "After Dead Instruction Elimination - ACIR".to_string(),
                ];
            }
            _ => (),
        }
    }

    // Disable some instructions with bugs that are not fixed yet
    let instruction_options = InstructionOptions {
        shl_enabled: false,
        shr_enabled: false,
        alloc_enabled: false,
        ..InstructionOptions::default()
    };
    let modes = vec![FuzzerMode::NonConstant, FuzzerMode::NonConstantWithoutDIE];
    let options =
        FuzzerOptions { compile_options, instruction_options, modes, ..FuzzerOptions::default() };
    let fuzzer_data = borrow_decode_from_slice(data, bincode::config::legacy())
        .unwrap_or((FuzzerData::default(), 1337))
        .0;
    let start = std::time::Instant::now();
    let fuzzer_output = fuzz_target(fuzzer_data, options);

    // If REDIS_URL is set and generated program is executed
    if redis::ensure_redis_connection() && fuzzer_output.is_some() {
        // cargo-fuzz saves tests with name equal to sha1 of content
        let fuzzer_output = fuzzer_output.unwrap();
        let mut hasher = Sha1::new();
        hasher.update(data);
        let sha1_hash = hasher.finalize();
        let test_id = format!("{sha1_hash:x}");
        match push_fuzzer_output_to_redis_queue("fuzzer_output", test_id, fuzzer_output) {
            Ok(json_str) => log::debug!("{json_str}"),
            Err(e) => log::error!("Failed to push to Redis queue: {e}"),
        }
    }

    if start.elapsed().as_secs() > MAX_EXECUTION_TIME_TO_KEEP_IN_CORPUS {
        return Corpus::Reject;
    }
    Corpus::Keep
});

libfuzzer_sys::fuzz_mutator!(|data: &mut [u8], _size: usize, max_size: usize, seed: u32| {
    let mut rng = StdRng::seed_from_u64(seed as u64);
    let mut new_fuzzer_data: FuzzerData = borrow_decode_from_slice(data, bincode::config::legacy())
        .unwrap_or((FuzzerData::default(), 1337))
        .0;
    mutate(&mut new_fuzzer_data, &mut rng);
    let new_bytes = encode_to_vec(&new_fuzzer_data, bincode::config::legacy()).unwrap();
    if new_bytes.len() > max_size {
        return 0;
    }
    data[..new_bytes.len()].copy_from_slice(&new_bytes);
    new_bytes.len()
});
