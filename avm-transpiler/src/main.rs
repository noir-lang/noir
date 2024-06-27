#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use log::warn;
use std::env;
use std::fs;
use std::path::Path;

mod instructions;
mod opcodes;
mod transpile;
mod transpile_contract;
mod utils;

use transpile_contract::{CompiledAcirContractArtifact, TranspiledContractArtifact};

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let in_contract_artifact_path = &args[1];
    let out_transpiled_artifact_path = &args[2];
    let json_parse_error = format!(
        "Unable to parse json for: {in_contract_artifact_path}
    This is probably a stale json file with a different wire format.
    You might need to recompile the contract or delete the json file"
    );

    // Parse original (pre-transpile) contract.
    let contract_json = fs::read_to_string(Path::new(in_contract_artifact_path))
        .expect(&format!("Unable to read file: {in_contract_artifact_path}"));
    let raw_json_obj: serde_json::Value =
        serde_json::from_str(&contract_json).expect(&json_parse_error);

    // Skip if contract has "transpiled: true" flag!
    if let Some(serde_json::Value::Bool(true)) = raw_json_obj.get("transpiled") {
        warn!("Contract already transpiled. Skipping.");
        return;
    }

    // Backup the output file if it already exists.
    if Path::new(out_transpiled_artifact_path).exists() {
        std::fs::copy(
            Path::new(out_transpiled_artifact_path),
            Path::new(&(out_transpiled_artifact_path.clone() + ".bak")),
        )
        .expect(&format!("Unable to backup file: {out_transpiled_artifact_path}"));
    }

    // Parse json into contract object
    let contract: CompiledAcirContractArtifact =
        serde_json::from_str(&contract_json).expect(&json_parse_error);

    // Transpile contract to AVM bytecode
    let transpiled_contract = TranspiledContractArtifact::from(contract);
    let transpiled_json =
        serde_json::to_string(&transpiled_contract).expect("Unable to serialize json");
    fs::write(out_transpiled_artifact_path, transpiled_json).expect("Unable to write file");
}
