#![no_main]

mod avm_integration;
pub(crate) mod fuzz_lib;
mod mutations;
mod utils;

use avm_integration::{AvmComparisonResult, compare_with_avm};
use bincode::serde::{borrow_decode_from_slice, encode_to_vec};
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

const MAX_EXECUTION_TIME_TO_KEEP_IN_CORPUS: u64 = 10;
const INLINE_TYPE: FrontendInlineType = FrontendInlineType::Inline;
const BRILLIG_RUNTIME: RuntimeType = RuntimeType::Brillig(INLINE_TYPE);
const TARGET_RUNTIMES: [RuntimeType; 1] = [BRILLIG_RUNTIME];

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

    // You can disable some instructions with bugs that are not fixed yet
    let modes = vec![FuzzerMode::NonConstant];
    let instruction_options = InstructionOptions {
        array_get_enabled: false,
        array_set_enabled: false,
        ecdsa_secp256k1_enabled: false,
        ecdsa_secp256r1_enabled: false,
        ..InstructionOptions::default()
    };
    let fuzzer_command_options =
        FuzzerCommandOptions { loops_enabled: false, ..FuzzerCommandOptions::default() };
    let options = FuzzerOptions {
        compile_options,
        instruction_options,
        modes,
        fuzzer_command_options,
        ..FuzzerOptions::default()
    };
    let fuzzer_data = borrow_decode_from_slice(data, bincode::config::legacy())
        .unwrap_or((FuzzerData::default(), 1337))
        .0;
    let start = std::time::Instant::now();
    let fuzzer_output = fuzz_target(fuzzer_data, TARGET_RUNTIMES.to_vec(), options);

    match compare_with_avm(&fuzzer_output) {
        AvmComparisonResult::Match => {
            log::info!("AVM and Brillig outputs match");
        }
        AvmComparisonResult::Mismatch { brillig_outputs, avm_outputs } => {
            log::error!("AVM and Brillig outputs mismatch!");
            log::error!("Brillig outputs: {:?}", brillig_outputs);
            log::error!("AVM outputs: {:?}", avm_outputs);
            panic!("AVM vs Brillig mismatch detected");
        }
        AvmComparisonResult::TranspilerError(err) => {
            panic!("Transpiler error: {}", err);
        }
        AvmComparisonResult::SimulatorError(err) => {
            panic!("Simulator error: {}", err);
        }
        AvmComparisonResult::BrilligCompilationError(err) => {
            log::debug!("Brillig compilation error: {}", err);
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
