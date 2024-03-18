use log::warn;
use std::env;
use std::fs;
use std::path::Path;

mod instructions;
mod opcodes;
mod transpile;
mod transpile_contract;
mod utils;

use transpile_contract::{CompiledAcirContract, TranspiledContract};

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let in_contract_artifact_path = &args[1];
    let out_transpiled_artifact_path = &args[2];

    // Parse original (pre-transpile) contract
    let contract_json =
        fs::read_to_string(Path::new(in_contract_artifact_path)).expect("Unable to read file");
    let raw_json_obj: serde_json::Value =
        serde_json::from_str(&contract_json).expect("Unable to parse json");

    // Skip if contract has "transpiled: true" flag!
    if let Some(serde_json::Value::Bool(true)) = raw_json_obj.get("transpiled") {
        warn!("Contract already transpiled. Skipping.");
        return;
    }

    // Backup the original file
    std::fs::copy(
        Path::new(in_contract_artifact_path),
        Path::new(&(in_contract_artifact_path.clone() + ".bak")),
    )
    .expect("Unable to backup file");

    // Parse json into contract object
    let contract: CompiledAcirContract =
        serde_json::from_str(&contract_json).expect("Unable to parse json");

    // Transpile contract to AVM bytecode
    let transpiled_contract = TranspiledContract::from(contract);
    let transpiled_json =
        serde_json::to_string(&transpiled_contract).expect("Unable to serialize json");
    fs::write(out_transpiled_artifact_path, transpiled_json).expect("Unable to write file");
}
