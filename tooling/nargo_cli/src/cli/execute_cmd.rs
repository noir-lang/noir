use acvm::acir::native_types::WitnessMap;
use clap::Args;

use nargo::artifacts::debug::DebugArtifact;
use nargo::constants::PROVER_INPUT_FILE;
use nargo::errors::try_to_diagnose_runtime_error;
use nargo::ops::DefaultForeignCallExecutor;
use nargo::package::Package;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_abi::input_parser::{Format, InputValue};
use noirc_abi::InputMap;
use noirc_driver::{CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING};
use noirc_frontend::graph::CrateName;

use super::compile_cmd::compile_bin_package;
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
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;
    let target_dir = &workspace.target_directory_path();

    let (np_language, opcode_support) = backend.get_backend_info_or_default();
    for package in &workspace {
        let compiled_program = compile_bin_package(
            &workspace,
            package,
            &args.compile_options,
            np_language,
            &opcode_support,
        )?;

        let (return_value, solved_witness) =
            execute_program_and_decode(compiled_program, package, &args.prover_name)?;

        println!("[{}] Circuit witness successfully solved", package.name);
        if let Some(return_value) = return_value {
            println!("[{}] Circuit output: {return_value:?}", package.name);
        }
        if let Some(witness_name) = &args.witness_name {
            let witness_path = save_witness_to_dir(solved_witness, witness_name, target_dir)?;

            println!("[{}] Witness saved to {}", package.name, witness_path.display());
        }
    }
    Ok(())
}

fn execute_program_and_decode(
    program: CompiledProgram,
    package: &Package,
    prover_name: &str,
) -> Result<(Option<InputValue>, WitnessMap), CliError> {
    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(&package.root_dir, prover_name, Format::Toml, &program.abi)?;
    let solved_witness = execute_program(&program, &inputs_map)?;
    let public_abi = program.abi.public_abi();
    let (_, return_value) = public_abi.decode(&solved_witness)?;

    Ok((return_value, solved_witness))
}

pub(crate) fn execute_program(
    compiled_program: &CompiledProgram,
    inputs_map: &InputMap,
) -> Result<WitnessMap, CliError> {
    #[allow(deprecated)]
    let blackbox_solver = barretenberg_blackbox_solver::BarretenbergSolver::new();

    let initial_witness = compiled_program.abi.encode(inputs_map, None)?;

    let solved_witness_err = nargo::ops::execute_circuit(
        &compiled_program.circuit,
        initial_witness,
        &blackbox_solver,
        &mut DefaultForeignCallExecutor::new(true),
    );
    match solved_witness_err {
        Ok(solved_witness) => Ok(solved_witness),
        Err(err) => {
            let debug_artifact = DebugArtifact {
                debug_symbols: vec![compiled_program.debug.clone()],
                file_map: compiled_program.file_map.clone(),
                warnings: compiled_program.warnings.clone(),
            };

            if let Some(diagnostic) = try_to_diagnose_runtime_error(&err, &compiled_program.debug) {
                diagnostic.report(&debug_artifact, false);
            }

            Err(crate::errors::CliError::NargoError(err))
        }
    }
}
