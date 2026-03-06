//! Binary file to mutate and execute Brillig programs.
//!
//! Usage after calling binary:
//! 1) mutate <b64_data> -> prints b64 mutated data
//! 2) execute <b64_data> -> prints JSON string of the fuzzer output
//!
//! TODO(sn): add cli args to disable ops
mod fuzz_lib;
mod mutations;
mod utils;
use base64::{Engine as _, engine::general_purpose};
use fuzz_lib::{fuzz_target_lib::fuzz_target, fuzzer::FuzzerData, options::FuzzerOptions};
use mutations::mutate as mutate_fuzzer_data;
use rand::{SeedableRng, rngs::StdRng};
use rmp_serde::{decode::from_slice as decode_from_slice, encode::to_vec as encode_to_rmp_vec};
use utils::fuzzer_output_to_json;

const SEED: u64 = 1337;

/// Mutates the fuzzer data
///
/// # Arguments
///
/// * `b64_data` - The base64 encoded serialized fuzzer data
///
/// # Returns
///
/// * `Vec<u8>` - The mutated serialized fuzzer data in base64 encoded format
///
fn mutate(b64_data: &[u8]) -> String {
    let data = general_purpose::STANDARD.decode(b64_data).unwrap_or_default();
    let mut new_fuzzer_data: FuzzerData =
        decode_from_slice(&data).unwrap_or((FuzzerData::default(), 1337)).0;
    let mut rng = StdRng::seed_from_u64(SEED);
    mutate_fuzzer_data(&mut new_fuzzer_data, &mut rng);
    let new_bytes = encode_to_rmp_vec(&new_fuzzer_data).unwrap();
    general_purpose::STANDARD.encode(&new_bytes)
}

/// Executes the fuzzer data
///
/// # Arguments
///
/// * `b64_data` - The base64 encoded serialized fuzzer data
///
/// # Returns
///
/// * `String` - The JSON string of the fuzzer output from [``fuzzer_output_to_json``]
///
fn execute(b64_data: &[u8]) -> String {
    let data = general_purpose::STANDARD.decode(b64_data).unwrap_or_default();
    let fuzzer_data: FuzzerData =
        decode_from_slice(&data).unwrap_or((FuzzerData::default(), 1337)).0;
    let fuzzer_output = fuzz_target(fuzzer_data, FuzzerOptions::default());
    match fuzzer_output {
        Some(fuzzer_output) => fuzzer_output_to_json(fuzzer_output),
        None => String::new(),
    }
}

fn main() {
    use std::io;
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).expect("Failed to read from stdin");
    let user_input = user_input.trim();
    let parts = user_input.split_whitespace().collect::<Vec<&str>>();
    if parts.len() != 2 {
        println!("Usage: mutate|execute <b64_data>");
        return;
    }
    match parts[0] {
        "mutate" => println!("{}", mutate(parts[1].as_bytes())),
        "execute" => println!("{}", execute(parts[1].as_bytes())),
        _ => println!("Invalid command"),
    }
}
