//! Module for comparing Brillig output with Brillig-compatible Abstract VM output
use crate::fuzz_lib::fuzzer::FuzzerOutput;
use acvm::acir::circuit::Program;
use acvm::{AcirField, FieldElement};
use base64::Engine;
use sancov::Counters;
use std::collections::HashMap;
use std::time::Instant;

/// Function for transpiling Brillig bytecode to Abstract VM bytecode
/// The first argument is the Brillig bytecode
pub(crate) type TranspileBrilligBytecodeToAbstractVMBytecode =
    Box<dyn Fn(String) -> Result<String, String>>;

/// Function for executing Abstract VM bytecode
/// The first argument is the Abstract VM bytecode
/// The second argument is the inputs as strings
/// The return value is a tuple with the outputs and the coverage map
pub(crate) type ExecuteAbstractVMBytecode =
    Box<dyn Fn(String, Vec<String>) -> (Result<Vec<String>, String>, HashMap<String, u16>)>;

#[derive(Debug)]
pub(crate) enum AbstractVMComparisonResult {
    Match,
    Mismatch { brillig_outputs: Vec<FieldElement>, abstract_vm_outputs: Vec<FieldElement> },
    TranspilerError(String),
    SimulatorError(String),
    BrilligCompilationError(String),
}

fn register_external_coverage(coverage: HashMap<String, u16>) {
    static COUNTERS: Counters<1_000_000> = Counters::new();
    COUNTERS.register();

    for (key, value) in coverage {
        for _ in 0..value {
            COUNTERS.hash_increment(&key);
        }
    }
}

pub(crate) fn compare_with_abstract_vm(
    fuzzer_output: &FuzzerOutput,
    transpiler: &TranspileBrilligBytecodeToAbstractVMBytecode,
    simulator: &ExecuteAbstractVMBytecode,
) -> AbstractVMComparisonResult {
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
    log::debug!("Bytecode serialization time: {:?}", step_start.elapsed());

    let step_start = Instant::now();
    let abstract_vm_bytecode = match transpiler(bytecode) {
        Ok(bc) => bc,
        Err(e) => return AbstractVMComparisonResult::TranspilerError(e),
    };
    log::debug!("Transpiler call time: {:?}", step_start.elapsed());

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
    log::debug!("Input extraction time: {:?}", step_start.elapsed());

    let step_start = Instant::now();
    let (abstract_vm_outputs_result, abstract_vm_coverage) =
        simulator(abstract_vm_bytecode, inputs);
    log::debug!("Simulator call time: {:?}", step_start.elapsed());

    let (abstract_vm_outputs, abstract_vm_coverage) = match abstract_vm_outputs_result {
        Ok(outputs) => (
            outputs
                .iter()
                .map(|output| FieldElement::try_from_str(output).unwrap())
                .collect::<Vec<FieldElement>>(),
            abstract_vm_coverage,
        ),
        Err(e) => {
            // brillig execution failed, so we assume the match
            if brillig_outputs.is_empty() {
                register_external_coverage(abstract_vm_coverage);
                return AbstractVMComparisonResult::Match;
            }
            log::info!("Brillig outputs: {brillig_outputs:?}");
            return AbstractVMComparisonResult::SimulatorError(e);
        }
    };
    let step_start = Instant::now();
    register_external_coverage(abstract_vm_coverage);
    log::debug!("Coverage registration time: {:?}", step_start.elapsed());

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
