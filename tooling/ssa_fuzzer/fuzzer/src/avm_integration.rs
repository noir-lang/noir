use crate::fuzz_lib::fuzzer::FuzzerOutput;
use crate::utils::fuzzer_output_to_json;
use acvm::acir::circuit::Program;
use acvm::{AcirField, FieldElement};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Instant;

const TRANSPILER_URL: &str = "http://localhost:51447/transpile";
const SIMULATOR_URL: &str = "http://localhost:51446/execute";

lazy_static! {
    static ref CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::new();
}

#[derive(Serialize)]
struct TranspilerRequest {
    bytecode: String,
}

#[derive(Deserialize)]
struct TranspilerResponse {
    avm_bytecode: String,
}

#[derive(Serialize)]
struct SimulatorRequest {
    avm_bytecode: String,
    inputs: Vec<String>,
}

#[derive(Deserialize)]
struct SimulatorResponse {
    reverted: bool,
    outputs: Vec<String>,
    error: Option<String>,
}

#[derive(Debug)]
pub(crate) enum AvmComparisonResult {
    Match,
    Mismatch { brillig_outputs: Vec<FieldElement>, avm_outputs: Vec<FieldElement> },
    TranspilerError(String),
    SimulatorError(String),
    BrilligCompilationError(String),
}

pub(crate) fn compare_with_avm(fuzzer_output: &FuzzerOutput) -> AvmComparisonResult {
    let total_start = Instant::now();

    // Step 1: Get Brillig outputs for comparison
    let step_start = Instant::now();
    let brillig_outputs = fuzzer_output.get_return_witnesses();
    let bytecode = if let Some(program) = &fuzzer_output.program {
        let serialized = Program::serialize_program(&program.program);
        base64::engine::general_purpose::STANDARD.encode(serialized)
    } else {
        return AvmComparisonResult::BrilligCompilationError(
            "No bytecode found in program".to_string(),
        );
    };
    log::debug!("Step 1 - Bytecode serialization: {:?}", step_start.elapsed());

    // Step 2: Send bytecode to transpiler service
    let step_start = Instant::now();
    let avm_bytecode = match call_transpiler(&bytecode) {
        Ok(bc) => bc,
        Err(e) => return AvmComparisonResult::TranspilerError(e),
    };
    log::debug!("Step 2 - Transpiler call: {:?}", step_start.elapsed());

    // TODO(sn): now simulator service perceives first input as a selector, which must fit in 32 bits
    if fuzzer_output.get_input_witnesses()[0].num_bits() >= 32 {
        return AvmComparisonResult::Match;
    }

    // Step 3: Extract inputs
    let step_start = Instant::now();
    let inputs = fuzzer_output
        .get_input_witnesses()
        .iter()
        .map(FieldElement::to_string)
        .collect::<Vec<String>>();
    log::debug!("Step 3 - Input extraction: {:?}", step_start.elapsed());

    // Step 5: Send to simulator service
    let avm_outputs = match call_simulator(&avm_bytecode, &inputs) {
        Ok(outputs) => outputs,
        Err(e) => {
            // brillig execution failed, so we assume the match
            if brillig_outputs.is_empty() {
                return AvmComparisonResult::Match;
            }
            return AvmComparisonResult::SimulatorError(e);
        }
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
    let total_start = Instant::now();
    let client = CLIENT.clone();
    log::debug!("Transpiler client created: {:?}", total_start.elapsed());
    let payload = TranspilerRequest { bytecode: bytecode.to_string() };

    let step_start = Instant::now();
    let response = client
        .post(TRANSPILER_URL)
        .json(&payload)
        .send()
        .map_err(|e| format!("Transpiler request failed: {e}"))?;

    log::debug!("Transpiler response: {:?}", step_start.elapsed());

    if !response.status().is_success() {
        return Err(format!("Transpiler returned status: {}", response.status()));
    }

    let step_start = Instant::now();
    let result: TranspilerResponse =
        response.json().map_err(|e| format!("Failed to parse transpiler response: {e}"))?;

    log::debug!("Transpiler response parsing: {:?}", step_start.elapsed());

    Ok(result.avm_bytecode)
}

fn call_simulator(avm_bytecode: &str, inputs: &[String]) -> Result<Vec<FieldElement>, String> {
    let client = CLIENT.clone();
    let payload =
        SimulatorRequest { avm_bytecode: avm_bytecode.to_string(), inputs: inputs.to_vec() };

    let step_start = Instant::now();
    let response = client
        .post(SIMULATOR_URL)
        .json(&payload)
        .send()
        .map_err(|e| format!("Simulator request failed: {e}"))?;

    log::debug!("Simulator response: {:?}", step_start.elapsed());

    if !response.status().is_success() {
        return Err(format!("Simulator returned status: {}", response.status()));
    }

    let step_start = Instant::now();
    let result: SimulatorResponse =
        response.json().map_err(|e| format!("Failed to parse simulator response: {e}"))?;

    if result.reverted {
        let error_msg = result.error.as_deref().unwrap_or("Unknown error");
        return Err(format!("AVM execution reverted: {error_msg}"));
    }

    let outputs = result
        .outputs
        .iter()
        .map(|hex_str| {
            // Convert hex string to FieldElement
            // The simulator returns hex strings, so we need to parse them
            FieldElement::from_hex(hex_str)
                .ok_or_else(|| format!("Failed to parse hex output: {hex_str}"))
        })
        .collect::<Result<Vec<FieldElement>, String>>()?;

    log::debug!("Simulator response parsing: {:?}", step_start.elapsed());

    Ok(outputs)
}
