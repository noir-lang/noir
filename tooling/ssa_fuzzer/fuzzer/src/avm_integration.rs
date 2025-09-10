use crate::fuzz_lib::fuzzer::FuzzerOutput;
use crate::utils::fuzzer_output_to_json;
use acvm::{AcirField, FieldElement};
use serde_json::{Value, json};

const TRANSPILER_URL: &str = "http://localhost:51447/transpile";
const SIMULATOR_URL: &str = "http://localhost:51446/execute";

#[derive(Debug)]
pub(crate) enum AvmComparisonResult {
    Match,
    Mismatch { brillig_outputs: Vec<FieldElement>, avm_outputs: Vec<FieldElement> },
    TranspilerError(String),
    SimulatorError(String),
    BrilligCompilationError(String),
}

pub(crate) fn compare_with_avm(fuzzer_output: &FuzzerOutput) -> AvmComparisonResult {
    // Get Brillig outputs for comparison
    let brillig_outputs = fuzzer_output.get_return_witnesses();

    // Convert fuzzer output to JSON format
    let json_output = fuzzer_output_to_json(fuzzer_output.clone());
    let parsed: Value = match serde_json::from_str(&json_output) {
        Ok(v) => v,
        Err(e) => {
            return AvmComparisonResult::TranspilerError(format!(
                "Failed to parse fuzzer output: {e}",
            ));
        }
    };

    // Extract bytecode from the JSON
    let bytecode = match parsed["program"]["bytecode"].as_str() {
        Some(bc) => bc,
        None => {
            return AvmComparisonResult::BrilligCompilationError(
                "No bytecode found in program".to_string(),
            );
        }
    };

    // Step 2: Send bytecode to transpiler service
    let avm_bytecode = match call_transpiler(bytecode) {
        Ok(bc) => bc,
        Err(e) => return AvmComparisonResult::TranspilerError(e),
    };

    // Step 4: Extract inputs from the JSON
    let inputs = match parsed["inputs"].as_array() {
        Some(inputs_array) => inputs_array
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
        None => vec![],
    };

    // Step 5: Send to simulator service
    let avm_outputs = match call_simulator(&avm_bytecode, &inputs) {
        Ok(outputs) => outputs,
        Err(e) => return AvmComparisonResult::SimulatorError(e),
    };

    // Step 7: Compare results
    if brillig_outputs.len() != avm_outputs.len() {
        return AvmComparisonResult::Mismatch { brillig_outputs, avm_outputs };
    }

    for (brillig_out, avm_out) in brillig_outputs.iter().zip(avm_outputs.iter()) {
        if *brillig_out != *avm_out {
            return AvmComparisonResult::Mismatch { brillig_outputs, avm_outputs };
        }
    }

    AvmComparisonResult::Match
}

fn call_transpiler(bytecode: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let payload = json!({
        "bytecode": bytecode
    });

    let response = client
        .post(TRANSPILER_URL)
        .json(&payload)
        .send()
        .map_err(|e| format!("Transpiler request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Transpiler returned status: {}", response.status()));
    }

    let result: Value =
        response.json().map_err(|e| format!("Failed to parse transpiler response: {e}"))?;

    match result["avm_bytecode"].as_str() {
        Some(avm_bytecode) => Ok(avm_bytecode.to_string()),
        None => Err("No avm_bytecode in transpiler response".to_string()),
    }
}

fn call_simulator(avm_bytecode: &str, inputs: &[String]) -> Result<Vec<FieldElement>, String> {
    let client = reqwest::blocking::Client::new();
    let payload = json!({
        "avm_bytecode": avm_bytecode,
        "inputs": inputs
    });

    let response = client
        .post(SIMULATOR_URL)
        .json(&payload)
        .send()
        .map_err(|e| format!("Simulator request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Simulator returned status: {}", response.status()));
    }

    let result: Value =
        response.json().map_err(|e| format!("Failed to parse simulator response: {e}"))?;

    if result["reverted"].as_bool().unwrap_or(false) {
        let error_msg = result["error"].as_str().unwrap_or("Unknown error");
        return Err(format!("AVM execution reverted: {error_msg}"));
    }

    let outputs = match result["outputs"].as_array() {
        Some(outputs_array) => {
            outputs_array
                .iter()
                .filter_map(|v| v.as_str())
                .map(|hex_str| {
                    // Convert hex string to FieldElement
                    // The simulator returns hex strings, so we need to parse them
                    FieldElement::from_hex(hex_str)
                        .ok_or_else(|| format!("Failed to parse hex output: {hex_str}"))
                })
                .collect::<Result<Vec<FieldElement>, String>>()?
        }
        None => vec![],
    };

    Ok(outputs)
}
