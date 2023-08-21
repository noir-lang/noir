use acvm::acir::circuit::OpcodeLocation;
use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm::pwg::ErrorLocation;
use acvm::Backend;
use clap::Args;
use nargo::constants::PROVER_INPUT_FILE;
use nargo::package::Package;
use nargo::NargoError;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_abi::input_parser::{Format, InputValue};
use noirc_abi::{Abi, InputMap};
use noirc_driver::{CompileOptions, CompiledProgram};
use noirc_errors::{debug_info::DebugInfo, CustomDiagnostic};
use noirc_frontend::graph::CrateName;
use noirc_frontend::hir::Context;

use super::compile_cmd::compile_package;
use super::fs::{inputs::read_inputs_from_file, witness::save_witness_to_dir};
use super::NargoConfig;
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

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: ExecuteCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(&toml_path, selection)?;
    let witness_dir = &workspace.target_directory_path();

    for package in &workspace {
        let (return_value, solved_witness) =
            execute_package(backend, package, &args.prover_name, &args.compile_options)?;

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

fn execute_package<B: Backend>(
    backend: &B,
    package: &Package,
    prover_name: &str,
    compile_options: &CompileOptions,
) -> Result<(Option<InputValue>, WitnessMap), CliError<B>> {
    let (context, compiled_program) = compile_package(backend, package, compile_options)?;
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

fn extract_opcode_error_from_nargo_error(
    nargo_err: &NargoError,
) -> Option<(OpcodeLocation, &acvm::pwg::OpcodeResolutionError)> {
    let solving_err = match nargo_err {
        nargo::NargoError::SolvingError(err) => err,
        _ => return None,
    };

    match solving_err {
        acvm::pwg::OpcodeResolutionError::IndexOutOfBounds {
            opcode_location: error_location,
            ..
        }
        | acvm::pwg::OpcodeResolutionError::UnsatisfiedConstrain {
            opcode_location: error_location,
        } => match error_location {
            ErrorLocation::Unresolved => {
                unreachable!("Cannot resolve index for unsatisfied constraint")
            }
            ErrorLocation::Resolved(opcode_location) => Some((*opcode_location, solving_err)),
        },
        _ => None,
    }
}

fn report_opcode_error(
    opcode_err_info: Option<(OpcodeLocation, &acvm::pwg::OpcodeResolutionError)>,
    debug: &DebugInfo,
    context: &Context,
) {
    if let Some((opcode_location, opcode_err)) = opcode_err_info {
        if let Some(locations) = debug.opcode_location(&opcode_location) {
            // The location of the error itself will be the location at the top
            // of the call stack (the last item in the Vec).
            if let Some(location) = locations.last() {
                match opcode_err {
                    acvm::pwg::OpcodeResolutionError::IndexOutOfBounds {
                        index,
                        array_size,
                        ..
                    } => {
                        let message = format!(
                            "Index out of bounds, array has size {array_size:?}, but index was {index:?}"
                        );
                        CustomDiagnostic::simple_error(message, String::new(), location.span)
                            .in_file(location.file)
                            .with_call_stack(locations)
                            .report(&context.file_manager, false);
                    }
                    acvm::pwg::OpcodeResolutionError::UnsatisfiedConstrain { .. } => {
                        let message = "Failed constraint".into();
                        CustomDiagnostic::simple_error(message, String::new(), location.span)
                            .in_file(location.file)
                            .with_call_stack(locations)
                            .report(&context.file_manager, false);
                    }
                    _ => (),
                }
            }
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
                let opcode_err_info = extract_opcode_error_from_nargo_error(&err);
                report_opcode_error(opcode_err_info, &debug, &context);
            }

            Err(crate::errors::CliError::NargoError(err))
        }
    }
}
