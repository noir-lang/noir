#![no_main]

pub(crate) mod fuzz_lib;
mod mutations;
mod utils;

use fuzz_lib::{
    fuzz_target_lib::fuzz_target,
    fuzzer::FuzzerData,
    options::{FuzzerCommandOptions, FuzzerMode, FuzzerOptions, InstructionOptions},
};
use libfuzzer_sys::Corpus;
use mutations::mutate;
use noirc_driver::CompileOptions;
use noirc_evaluator::ssa::ir::function::RuntimeType;
use noirc_frontend::monomorphization::ast::InlineType as FrontendInlineType;
use rand::{SeedableRng, rngs::StdRng};
use rmp_serde::{decode::from_slice as decode_from_slice, encode::to_vec as encode_to_rmp_vec};
use sha1::{Digest, Sha1};
use utils::{push_fuzzer_output_to_redis_queue, redis};

const MAX_EXECUTION_TIME_TO_KEEP_IN_CORPUS: u64 = 3;
const INLINE_TYPE: FrontendInlineType = FrontendInlineType::Inline;
const ACIR_RUNTIME: RuntimeType = RuntimeType::Acir(INLINE_TYPE);
const BRILLIG_RUNTIME: RuntimeType = RuntimeType::Brillig(INLINE_TYPE);
const TARGET_RUNTIMES: [RuntimeType; 2] = [ACIR_RUNTIME, BRILLIG_RUNTIME];

libfuzzer_sys::fuzz_target!(|data: &[u8]| -> Corpus {
    let _ = env_logger::try_init();

    let mut compile_options = CompileOptions::default();
    if let Ok(triage_value) = std::env::var("TRIAGE") {
        match triage_value.as_str() {
            "FULL" => compile_options.show_ssa = true,
            "FINAL" => {
                compile_options.show_ssa_pass =
                    vec!["Dead Instruction Elimination (3)".to_string()];
            }
            "FIRST_AND_FINAL" => {
                compile_options.show_ssa_pass = vec![
                    "After Removing Unreachable Functions (1)".to_string(),
                    "Dead Instruction Elimination (3)".to_string(),
                ];
            }
            _ => (),
        }
    }

    // Disable some instructions with bugs that are not fixed yet
    let instruction_options = InstructionOptions {
        // https://github.com/noir-lang/noir/issues/9437
        array_get_enabled: false,
        array_set_enabled: false,
        // https://github.com/noir-lang/noir/issues/9559
        point_add_enabled: false,
        multi_scalar_mul_enabled: false,
        // https://github.com/noir-lang/noir/issues/10037
        ecdsa_secp256k1_enabled: false,
        ecdsa_secp256r1_enabled: false,
        ..InstructionOptions::default()
    };
    let modes = vec![FuzzerMode::NonConstant];
    let fuzzer_command_options =
        FuzzerCommandOptions { loops_enabled: false, ..FuzzerCommandOptions::default() };
    let options = FuzzerOptions {
        compile_options,
        instruction_options,
        modes,
        fuzzer_command_options,
        ..FuzzerOptions::default()
    };
    let fuzzer_data = decode_from_slice(data).unwrap_or((FuzzerData::default(), 1337)).0;
    let start = std::time::Instant::now();
    let fuzzer_output = fuzz_target(fuzzer_data, TARGET_RUNTIMES.to_vec(), options);

    // If REDIS_URL is set and generated program is executed
    if redis::ensure_redis_connection() {
        // cargo-fuzz saves tests with name equal to sha1 of content
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
    let mut rng = StdRng::seed_from_u64(u64::from(seed));
    let mut new_fuzzer_data: FuzzerData =
        decode_from_slice(data).unwrap_or((FuzzerData::default(), 1337)).0;
    mutate(&mut new_fuzzer_data, &mut rng);
    let new_bytes = encode_to_rmp_vec(&new_fuzzer_data).unwrap();
    if new_bytes.len() > max_size {
        return 0;
    }
    data[..new_bytes.len()].copy_from_slice(&new_bytes);
    new_bytes.len()
});
