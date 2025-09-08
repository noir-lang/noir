use std::path::Path;

use acir::{AcirField, FieldElement, native_types::WitnessStack};
use acvm::BlackBoxFunctionSolver;
use nargo::{NargoError, foreign_calls::ForeignCallExecutor};
use noirc_abi::{AbiType, Sign, input_parser::InputValue};
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
pub fn show_diagnostic(circuit: &CompiledProgram, err: &NargoError<FieldElement>) {
    if let Some(diagnostic) =
        nargo::errors::try_to_diagnose_runtime_error(err, &circuit.abi, &circuit.debug)
    {
        let debug_artifact = DebugArtifact {
            debug_symbols: circuit.debug.clone(),
            file_map: circuit.file_map.clone(),
        };

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
    noirc_errors::println_to_stdout!("[{circuit_name}] Circuit witness successfully solved");
    // Save first, so that we can potentially look at the output if the expectations fail.
    if let Some(witness_dir) = witness_dir {
        save_witness(&results.witness_stack, circuit_name, witness_dir, witness_name)?;
    }
    if let Some(ref return_value) = results.return_values.actual_return {
        let abi_type = &circuit.abi.return_type.as_ref().unwrap().abi_type;
        let output_string = input_value_to_string(return_value, abi_type);
        noirc_errors::println_to_stdout!("[{circuit_name}] Circuit output: {output_string}");
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
    let mut witness_path = save_witness_to_dir(witness_stack, witness_name, witness_dir)?;

    // See if we can make the file path a bit shorter/easier to read if it starts with the current directory
    if let Ok(current_dir) = std::env::current_dir() {
        if let Ok(name_without_prefix) = witness_path.strip_prefix(current_dir) {
            witness_path = name_without_prefix.to_path_buf();
        }
    }

    noirc_errors::println_to_stdout!(
        "[{}] Witness saved to {}",
        circuit_name,
        witness_path.display()
    );
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

pub fn input_value_to_string(input_value: &InputValue, abi_type: &AbiType) -> String {
    let mut string = String::new();
    append_input_value_to_string(input_value, abi_type, &mut string);
    string
}

fn append_input_value_to_string(input_value: &InputValue, abi_type: &AbiType, string: &mut String) {
    match (abi_type, input_value) {
        (AbiType::Field, InputValue::Field(field_element)) => {
            string.push_str(&field_element.to_short_hex());
        }
        (AbiType::Array { length: _, typ }, InputValue::Vec(input_values)) => {
            string.push('[');
            for (index, input_value) in input_values.iter().enumerate() {
                if index != 0 {
                    string.push_str(", ");
                }
                append_input_value_to_string(input_value, typ, string);
            }
            string.push(']');
        }
        (AbiType::Integer { sign, width: bit_size }, InputValue::Field(f)) => match sign {
            Sign::Unsigned => {
                string.push_str(&f.to_string());
            }
            Sign::Signed => {
                string.push_str(&f.to_string_as_signed_integer(*bit_size));
            }
        },
        (AbiType::Boolean, InputValue::Field(field_element)) => {
            if field_element.is_zero() {
                string.push_str("false");
            } else {
                string.push_str("true");
            }
        }
        (AbiType::Struct { path, fields: field_types }, InputValue::Struct(field_values)) => {
            string.push_str(path);
            string.push_str(" { ");
            for (index, (field_name, field_value)) in field_values.iter().enumerate() {
                if index != 0 {
                    string.push_str(", ");
                }
                string.push_str(field_name);
                string.push_str(": ");
                let typ = &field_types.iter().find(|(name, _)| name == field_name).unwrap().1;
                append_input_value_to_string(field_value, typ, string);
            }
            string.push_str(" }");
        }
        (AbiType::Tuple { fields }, InputValue::Vec(input_values)) => {
            assert_eq!(fields.len(), input_values.len());

            string.push('(');
            for (index, (input_value, field_type)) in input_values.iter().zip(fields).enumerate() {
                if index != 0 {
                    string.push_str(", ");
                }
                append_input_value_to_string(input_value, field_type, string);
            }
            if input_values.len() == 1 {
                string.push(',');
            }
            string.push(')');
        }
        (AbiType::String { .. }, InputValue::String(value)) => {
            string.push_str(&format!("{value:?}"));
        }
        (_, _) => {
            panic!("Unexpected InputValue-AbiType combination: {input_value:?} - {abi_type:?}");
        }
    }
}
