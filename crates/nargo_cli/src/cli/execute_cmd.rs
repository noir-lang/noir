use std::path::Path;

use acvm::acir::circuit::OpcodeLabel;
use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm::Backend;
use clap::Args;
use nargo::NargoError;
use noirc_abi::input_parser::{Format, InputValue};
use noirc_abi::{Abi, InputMap};
use noirc_driver::{CompileOptions, CompiledProgram};
use noirc_errors::{debug_info::DebugInfo, CustomDiagnostic};
use noirc_frontend::hir::Context;

use super::fs::{inputs::read_inputs_from_file, witness::save_witness_to_dir};
use super::NargoConfig;
use crate::{
    cli::compile_cmd::compile_circuit,
    constants::{PROVER_INPUT_FILE, TARGET_DIR},
    errors::CliError,
};

/// Executes a circuit to calculate its return value
#[derive(Debug, Clone, Args)]
pub(crate) struct ExecuteCommand {
    /// Write the execution witness to named file
    witness_name: Option<String>,

    /// The name of the toml file which contains the inputs for the prover
    #[clap(long, short, default_value = PROVER_INPUT_FILE)]
    prover_name: String,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: ExecuteCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let (return_value, solved_witness) =
        execute_with_path(backend, &config.program_dir, args.prover_name, &args.compile_options)?;

    println!("Circuit witness successfully solved");
    if let Some(return_value) = return_value {
        println!("Circuit output: {return_value:?}");
    }
    if let Some(witness_name) = args.witness_name {
        let witness_dir = config.program_dir.join(TARGET_DIR);

        let witness_path = save_witness_to_dir(solved_witness, &witness_name, witness_dir)?;

        println!("Witness saved to {}", witness_path.display());
    }
    Ok(())
}

fn execute_with_path<B: Backend>(
    backend: &B,
    program_dir: &Path,
    prover_name: String,
    compile_options: &CompileOptions,
) -> Result<(Option<InputValue>, WitnessMap), CliError<B>> {
    let (compiled_program, context) = compile_circuit(backend, None, program_dir, compile_options)?;
    let CompiledProgram { abi, circuit, debug } = compiled_program;

    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(program_dir, prover_name.as_str(), Format::Toml, &abi)?;

    let solved_witness =
        execute_program(backend, circuit, &abi, &inputs_map, Some((debug, context)))?;
    let public_abi = abi.public_abi();
    let (_, return_value) = public_abi.decode(&solved_witness)?;

    Ok((return_value, solved_witness))
}

fn extract_unsatisfied_constraint_from_nargo_error(nargo_err: &NargoError) -> Option<usize> {
    let solving_err = match nargo_err {
        nargo::NargoError::SolvingError(err) => err,
        _ => return None,
    };

    match solving_err {
        acvm::pwg::OpcodeResolutionError::UnsatisfiedConstrain { opcode_label } => {
            match opcode_label {
                OpcodeLabel::Unresolved => {
                    unreachable!("Cannot resolve index for unsatisfied constraint")
                }
                OpcodeLabel::Resolved(opcode_index) => Some(*opcode_index as usize),
            }
        }
        _ => None,
    }
}
fn report_unsatisfied_constraint_error(
    opcode_idx: Option<usize>,
    debug: &DebugInfo,
    context: &Context,
) {
    if let Some(opcode_index) = opcode_idx {
        if let Some(loc) = debug.opcode_location(opcode_index) {
            noirc_errors::reporter::report(
                &context.file_manager,
                &CustomDiagnostic::simple_error(
                    "Unsatisfied constraint".to_string(),
                    "Constraint failed".to_string(),
                    loc.span,
                ),
                Some(loc.file),
                false,
            );
        }
    }
}

pub(crate) fn execute_program<B: Backend>(
    backend: &B,
    circuit: Circuit,
    abi: &Abi,
    inputs_map: &InputMap,
    debug_data: Option<(DebugInfo, Context)>,
) -> Result<WitnessMap, CliError<B>> {
    let initial_witness = abi.encode(inputs_map, None)?;
    let solved_witness_err = nargo::ops::execute_circuit(backend, circuit, initial_witness, true);
    match solved_witness_err {
        Ok(solved_witness) => Ok(solved_witness),
        Err(err) => {
            if let Some((debug, context)) = debug_data {
                let opcode_idx = extract_unsatisfied_constraint_from_nargo_error(&err);
                report_unsatisfied_constraint_error(opcode_idx, &debug, &context);
            }

            Err(crate::errors::CliError::NargoError(err))
        }
    }
}
