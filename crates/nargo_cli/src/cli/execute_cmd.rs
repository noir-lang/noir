use acvm::acir::circuit::{Opcode, OpcodeLocation};
use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm::pwg::{ErrorLocation, OpcodeResolutionError};
use acvm::Language;
use clap::Args;
use fm::FileManager;
use nargo::constants::PROVER_INPUT_FILE;
use nargo::errors::{ExecutionError, NargoError};
use nargo::package::Package;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_abi::input_parser::{Format, InputValue};
use noirc_abi::{Abi, InputMap};
use noirc_driver::{CompileOptions, CompiledProgram};
use noirc_errors::{debug_info::DebugInfo, CustomDiagnostic};
use noirc_frontend::graph::CrateName;

use super::compile_cmd::compile_package;
use super::fs::{inputs::read_inputs_from_file, witness::save_witness_to_dir};
use super::NargoConfig;
use crate::backends::Backend;
use crate::errors::CliError;

/// Executes a circuit to calculate its return value
#[derive(Debug, Clone, Args)]
pub(crate) struct ExecuteCommand {
    /// Write the execution witness to named file
    witness_name: Option<String>,

    /// The name of the toml file which contains the inputs for the prover
    #[clap(long, short, default_value = PROVER_INPUT_FILE)]
    prover_name: String,

    /// The name of the package to execute
    #[clap(long, conflicts_with = "workspace")]
    package: Option<CrateName>,

    /// Execute all packages in the workspace
    #[clap(long, conflicts_with = "package")]
    workspace: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(
    backend: &Backend,
    args: ExecuteCommand,
    config: NargoConfig,
) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(&toml_path, selection)?;
    let witness_dir = &workspace.target_directory_path();

    let (np_language, is_opcode_supported) = backend.get_backend_info()?;
    for package in &workspace {
        let (return_value, solved_witness) = execute_package(
            backend,
            package,
            &args.prover_name,
            &args.compile_options,
            np_language,
            &is_opcode_supported,
        )?;

        println!("[{}] Circuit witness successfully solved", package.name);
        if let Some(return_value) = return_value {
            println!("[{}] Circuit output: {return_value:?}", package.name);
        }
        if let Some(witness_name) = &args.witness_name {
            let witness_path = save_witness_to_dir(solved_witness, witness_name, witness_dir)?;

            println!("[{}] Witness saved to {}", package.name, witness_path.display());
        }
    }
    Ok(())
}

fn execute_package(
    backend: &Backend,
    package: &Package,
    prover_name: &str,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> Result<(Option<InputValue>, WitnessMap), CliError> {
    let (context, compiled_program) =
        compile_package(package, compile_options, np_language, &is_opcode_supported)?;
    let CompiledProgram { abi, circuit, debug } = compiled_program;

    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(&package.root_dir, prover_name, Format::Toml, &abi)?;
    let solved_witness =
        execute_program(backend, circuit, &abi, &inputs_map, Some((debug, context)))?;
    let public_abi = abi.public_abi();
    let (_, return_value) = public_abi.decode(&solved_witness)?;

    Ok((return_value, solved_witness))
}

/// There are certain errors that contain an [acvm::pwg::ErrorLocation].
/// We need to determine whether the error location has been resolving during execution.
/// If the location has been resolved we return the contained [OpcodeLocation].
fn extract_opcode_error_from_nargo_error(
    nargo_err: &NargoError,
) -> Option<(Vec<OpcodeLocation>, &ExecutionError)> {
    let execution_error = match nargo_err {
        NargoError::ExecutionError(err) => err,
        _ => return None,
    };

    match execution_error {
        ExecutionError::SolvingError(OpcodeResolutionError::BrilligFunctionFailed {
            call_stack,
            ..
        })
        | ExecutionError::AssertionFailed(_, call_stack) => {
            Some((call_stack.clone(), execution_error))
        }
        ExecutionError::SolvingError(OpcodeResolutionError::IndexOutOfBounds {
            opcode_location: error_location,
            ..
        })
        | ExecutionError::SolvingError(OpcodeResolutionError::UnsatisfiedConstrain {
            opcode_location: error_location,
        }) => match error_location {
            ErrorLocation::Unresolved => {
                unreachable!("Cannot resolve index for unsatisfied constraint")
            }
            ErrorLocation::Resolved(opcode_location) => {
                Some((vec![*opcode_location], execution_error))
            }
        },
        _ => None,
    }
}

/// Resolve the vector of [OpcodeLocation] that caused an execution error using the debug information
/// generated during compilation to determine the complete call stack for an error. Then report the error using
/// the resolved call stack and any other relevant error information returned from the ACVM.
fn report_error_with_opcode_locations(
    opcode_err_info: Option<(Vec<OpcodeLocation>, &ExecutionError)>,
    debug: &DebugInfo,
    file_manager: &FileManager,
) {
    if let Some((opcode_locations, opcode_err)) = opcode_err_info {
        let source_locations: Vec<_> = opcode_locations
            .iter()
            .flat_map(|opcode_location| {
                let locations = debug.opcode_location(opcode_location);
                locations.unwrap_or_default()
            })
            .collect();
        // The location of the error itself will be the location at the top
        // of the call stack (the last item in the Vec).
        if let Some(location) = source_locations.last() {
            let message = match opcode_err {
                ExecutionError::AssertionFailed(message, _) => {
                    format!("Assertion failed: '{message}'")
                }
                ExecutionError::SolvingError(OpcodeResolutionError::IndexOutOfBounds {
                    index,
                    array_size,
                    ..
                }) => {
                    format!(
                            "Index out of bounds, array has size {array_size:?}, but index was {index:?}"
                        )
                }
                ExecutionError::SolvingError(OpcodeResolutionError::UnsatisfiedConstrain {
                    ..
                }) => "Failed constraint".into(),
                _ => {
                    // All other errors that do not have corresponding opcode locations
                    // should not be reported in this method.
                    // If an error with an opcode location is not handled in this match statement
                    // the basic message attached to the original error from the ACVM should be reported.
                    return;
                }
            };
            CustomDiagnostic::simple_error(message, String::new(), location.span)
                .in_file(location.file)
                .with_call_stack(source_locations)
                .report(file_manager.as_file_map(), false);
        }
    }
}

pub(crate) fn execute_program(
    _backend: &Backend,
    circuit: Circuit,
    abi: &Abi,
    inputs_map: &InputMap,
    debug_data: Option<(DebugInfo, FileManager)>,
) -> Result<WitnessMap, CliError> {
    #[allow(deprecated)]
    let blackbox_solver = acvm::blackbox_solver::BarretenbergSolver::new();

    let initial_witness = abi.encode(inputs_map, None)?;

    let solved_witness_err =
        nargo::ops::execute_circuit(&blackbox_solver, circuit, initial_witness, true);
    match solved_witness_err {
        Ok(solved_witness) => Ok(solved_witness),
        Err(err) => {
            if let Some((debug, file_manager)) = debug_data {
                let opcode_err_info = extract_opcode_error_from_nargo_error(&err);
                report_error_with_opcode_locations(opcode_err_info, &debug, &file_manager);
            }

            Err(crate::errors::CliError::NargoError(err))
        }
    }
}
