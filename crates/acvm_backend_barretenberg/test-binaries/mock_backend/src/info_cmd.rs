use clap::Args;
use std::io::Write;
use std::path::PathBuf;

const INFO_RESPONSE: &str = r#"{
    "language": {
        "name": "PLONK-CSAT",
        "width": 3
    },
    "opcodes_supported": ["arithmetic", "directive", "brillig", "memory_init", "memory_op"],
    "black_box_functions_supported": [
        "and",
        "xor",
        "range",
        "sha256",
        "blake2s",
        "schnorr_verify",
        "pedersen",
        "hash_to_field_128_security",
        "ecdsa_secp256k1",
        "ecdsa_secp256r1",
        "fixed_base_scalar_mul",
        "recursive_aggregation"
    ]
}"#;

#[derive(Debug, Clone, Args)]
pub(crate) struct InfoCommand {
    #[clap(short = 'c')]
    pub(crate) crs_path: Option<PathBuf>,
}

pub(crate) fn run(_args: InfoCommand) {
    std::io::stdout().write_all(INFO_RESPONSE.as_bytes()).unwrap();
}
