use crate::fuzz_lib::fuzzer::FuzzerOutput;
use acvm::acir::circuit::Program;
use acvm::{AcirField, FieldElement};
use base64::Engine;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Instant;

lazy_static! {
    static ref ABSTRACT_VM_TRANSPILER_URL: String =
        env::var("TRANSPILER_URL").unwrap_or("http://localhost:51447/transpile".to_string());
    static ref ABSTRACT_VM_SIMULATOR_URL: String =
        env::var("SIMULATOR_URL").unwrap_or("http://localhost:51446/execute".to_string());
    static ref CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::new();
}

#[derive(Serialize)]
struct TranspilerRequest {
    bytecode: String,
}

#[derive(Deserialize)]
struct TranspilerResponse {
    abstract_vm_bytecode: String,
}

#[derive(Serialize)]
struct SimulatorRequest {
    abstract_vm_bytecode: String,
    inputs: Vec<String>,
}

#[derive(Deserialize)]
struct SimulatorResponse {
    reverted: bool,
    outputs: Vec<String>,
    error: Option<String>,
}

#[derive(Debug)]
pub(crate) enum AbstractVMComparisonResult {
    Match,
    Mismatch { brillig_outputs: Vec<FieldElement>, abstract_vm_outputs: Vec<FieldElement> },
    TranspilerError(String),
    SimulatorError(String),
    BrilligCompilationError(String),
}

pub(crate) fn compare_with_abstract_vm(fuzzer_output: &FuzzerOutput) -> AbstractVMComparisonResult {
    let step_start = Instant::now();
    let brillig_outputs = fuzzer_output.get_return_witnesses();
    let bytecode = if let Some(program) = &fuzzer_output.program {
        let serialized = Program::serialize_program(&program.program);
        base64::engine::general_purpose::STANDARD.encode(serialized)
    } else {
        return AbstractVMComparisonResult::BrilligCompilationError(
            "No bytecode found in program".to_string(),
        );
    };
    log::debug!("Bytecode serialization: {:?}", step_start.elapsed());

    let step_start = Instant::now();
    let abstract_vm_bytecode = match call_transpiler(&bytecode) {
        Ok(bc) => bc,
        Err(e) => return AbstractVMComparisonResult::TranspilerError(e),
    };
    log::debug!("Transpiler call: {:?}", step_start.elapsed());

    // TODO(sn): now simulator service perceives first input as a selector, which must fit in 32 bits
    if fuzzer_output.get_input_witnesses()[0].num_bits() >= 32 {
        return AbstractVMComparisonResult::Match;
    }

    let step_start = Instant::now();
    let inputs = fuzzer_output
        .get_input_witnesses()
        .iter()
        .map(FieldElement::to_string)
        .collect::<Vec<String>>();
    log::debug!("Input extraction: {:?}", step_start.elapsed());

    let abstract_vm_outputs = match call_simulator(&abstract_vm_bytecode, &inputs) {
        Ok(outputs) => outputs,
        Err(e) => {
            // brillig execution failed, so we assume the match
            if brillig_outputs.is_empty() {
                return AbstractVMComparisonResult::Match;
            }
            return AbstractVMComparisonResult::SimulatorError(e);
        }
    };

    if brillig_outputs.len() != abstract_vm_outputs.len() {
        return AbstractVMComparisonResult::Mismatch { brillig_outputs, abstract_vm_outputs };
    }

    for (brillig_out, abstract_vm_out) in brillig_outputs.iter().zip(abstract_vm_outputs.iter()) {
        if *brillig_out != *abstract_vm_out {
            return AbstractVMComparisonResult::Mismatch { brillig_outputs, abstract_vm_outputs };
        }
    }

    AbstractVMComparisonResult::Match
}

fn call_transpiler(bytecode: &str) -> Result<String, String> {
    let client = CLIENT.clone();
    let payload = TranspilerRequest { bytecode: bytecode.to_string() };

    let step_start = Instant::now();
    let response = client
        .post(ABSTRACT_VM_TRANSPILER_URL.as_str())
        .json(&payload)
        .send()
        .map_err(|e| format!("Transpiler request failed: {e}"))?;

    log::debug!("Transpiler response time: {:?}", step_start.elapsed());

    if !response.status().is_success() {
        return Err(format!("Transpiler returned status: {}", response.status()));
    }

    let step_start = Instant::now();
    let result: TranspilerResponse =
        response.json().map_err(|e| format!("Failed to parse transpiler response: {e}"))?;

    log::debug!("Transpiler response parsing: {:?}", step_start.elapsed());

    Ok(result.abstract_vm_bytecode)
}

fn call_simulator(
    abstract_vm_bytecode: &str,
    inputs: &[String],
) -> Result<Vec<FieldElement>, String> {
    let client = CLIENT.clone();
    let payload = SimulatorRequest {
        abstract_vm_bytecode: abstract_vm_bytecode.to_string(),
        inputs: inputs.to_vec(),
    };

    let step_start = Instant::now();
    let response = client
        .post(ABSTRACT_VM_SIMULATOR_URL.as_str())
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
