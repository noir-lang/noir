use std::path::Path;

use acir::{native_types::WitnessStack, FieldElement};
use acvm::BlackBoxFunctionSolver;
use nargo::{foreign_calls::ForeignCallExecutor, NargoError};
use noirc_abi::input_parser::InputValue;
use noirc_artifacts::debug::DebugArtifact;
use noirc_driver::CompiledProgram;

use crate::{
    errors::CliError,
    fs::{inputs::read_inputs_from_file, witness::save_witness_to_dir},
};

/// Results of a circuit execution.
#[derive(Clone, Debug)]
pub struct ExecutionResults {
    pub witness_stack: WitnessStack<FieldElement>,
    pub return_values: ReturnValues,
}

/// The decoded `return` witnesses.
#[derive(Clone, Debug)]
pub struct ReturnValues {
    /// The `return` value from the `Prover.toml` file, if present.
    pub expected_return: Option<InputValue>,
    /// The `return` value from the circuit execution.
    pub actual_return: Option<InputValue>,
}

/// Execute a circuit and return the output witnesses.
pub fn execute<B, E>(
    circuit: &CompiledProgram,
    blackbox_solver: &B,
    foreign_call_executor: &mut E,
    prover_file: &Path,
) -> Result<ExecutionResults, CliError>
where
    B: BlackBoxFunctionSolver<FieldElement>,
    E: ForeignCallExecutor<FieldElement>,
{
    let (input_map, expected_return) = read_inputs_from_file(prover_file, &circuit.abi)?;

    let initial_witness = circuit.abi.encode(&input_map, None)?;

    let witness_stack = nargo::ops::execute_program(
        &circuit.program,
        initial_witness,
        blackbox_solver,
        foreign_call_executor,
    )?;

    let main_witness =
        &witness_stack.peek().expect("Should have at least one witness on the stack").witness;

    let (_, actual_return) = circuit.abi.decode(main_witness)?;

    Ok(ExecutionResults {
        witness_stack,
        return_values: ReturnValues { actual_return, expected_return },
    })
}

/// Print an error stack trace, if possible.
pub fn show_diagnostic(circuit: CompiledProgram, err: NargoError<FieldElement>) {
    if let Some(diagnostic) =
        nargo::errors::try_to_diagnose_runtime_error(&err, &circuit.abi, &circuit.debug)
    {
        let debug_artifact =
            DebugArtifact { debug_symbols: circuit.debug, file_map: circuit.file_map };

        diagnostic.report(&debug_artifact, false);
    }
}

/// Print some information and save the witness if an output directory is specified,
/// then checks if the expected return values were the ones we expected.
pub fn save_and_check_witness(
    circuit: &CompiledProgram,
    results: ExecutionResults,
    circuit_name: &str,
    witness_dir: Option<&Path>,
    witness_name: Option<&str>,
) -> Result<(), CliError> {
    println!("[{}] Circuit witness successfully solved", circuit_name);
    // Save first, so that we can potentially look at the output if the expectations fail.
    if let Some(witness_dir) = witness_dir {
        save_witness(&results.witness_stack, circuit_name, witness_dir, witness_name)?;
    }
    check_witness(circuit, results.return_values)
}

/// Save the witness stack to a file.
pub fn save_witness(
    witness_stack: &WitnessStack<FieldElement>,
    circuit_name: &str,
    witness_dir: &Path,
    witness_name: Option<&str>,
) -> Result<(), CliError> {
    let witness_name = witness_name.unwrap_or(circuit_name);
    let witness_path = save_witness_to_dir(witness_stack, witness_name, witness_dir)?;
    println!("[{}] Witness saved to {}", circuit_name, witness_path.display());
    Ok(())
}

/// Compare return values to expectations, returning errors if something unexpected was returned.
pub fn check_witness(
    circuit: &CompiledProgram,
    return_values: ReturnValues,
) -> Result<(), CliError> {
    // Check that the circuit returned a non-empty result if the ABI expects a return value.
    if let Some(ref expected) = circuit.abi.return_type {
        if return_values.actual_return.is_none() {
            return Err(CliError::MissingReturn { expected: expected.clone() });
        }
    }

    // Check that if the prover file contained a `return` entry then that's what we got.
    if let Some(expected) = return_values.expected_return {
        match return_values.actual_return {
            None => {
                return Err(CliError::UnexpectedReturn { expected, actual: None });
            }
            Some(actual) => {
                if actual != expected {
                    return Err(CliError::UnexpectedReturn { expected, actual: Some(actual) });
                }
            }
        }
    }

    Ok(())
}
