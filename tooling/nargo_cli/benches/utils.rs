use std::path::PathBuf;

const BLACKBOX_FUNCTIONS: &[&str] = &[
    "blake2s",
    "blake3",
    "bit_and",
    "ecdsa_secp256k1",
    "ecdsa_secp256r1",
    "keccak256",
    "pedersen_commitment",
    "pedersen_hash",
    "scalar_mul",
    "schnorr",
    "xor",
];

const CRYPTOGRAPHIC_PRIMITIVES: &[&str] =
    &["eddsa", "poseidon_bn254_hash", "merkle_insert", "sha256"];

const RECURSION: &[&str] = &["double_verify_proof", "double_verify_nested_proof"];

#[allow(unused)]
fn get_selected_tests() -> Vec<PathBuf> {
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => std::env::current_dir().unwrap(),
    };
    let test_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test_programs")
        .join("execution_success");

    let selected_tests = BLACKBOX_FUNCTIONS.iter().chain(CRYPTOGRAPHIC_PRIMITIVES).chain(RECURSION);

    selected_tests.map(|t| test_dir.join(t)).collect()
}
