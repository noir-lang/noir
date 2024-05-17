use acvm::acir::native_types::WitnessStack;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
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

use super::compile_cmd::compile_workspace_full;
use super::fs::{inputs::read_inputs_from_file, witness::save_witness_to_dir};
use super::NargoConfig;
use crate::cli::fs::program::read_program_from_file;
use crate::errors::CliError;

/// Executes a circuit to calculate its return value
#[derive(Debug, Clone, Args)]
#[clap(visible_alias = "e")]
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

    /// JSON RPC url to solve oracle calls
    #[clap(long)]
    oracle_resolver: Option<String>,
}

pub(crate) fn run(args: ExecuteCommand, config: NargoConfig) -> Result<(), CliError> {
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

    // Compile the full workspace in order to generate any build artifacts.
    compile_workspace_full(&workspace, &args.compile_options)?;

    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());
    for package in binary_packages {
        let program_artifact_path = workspace.package_build_path(package);
        let program: CompiledProgram = read_program_from_file(program_artifact_path)?.into();

        let (return_value, witness_stack) = execute_program_and_decode(
            program,
            package,
            &args.prover_name,
            args.oracle_resolver.as_deref(),
        )?;

        println!("[{}] Circuit witness successfully solved", package.name);
        if let Some(return_value) = return_value {
            println!("[{}] Circuit output: {return_value:?}", package.name);
        }
        if let Some(witness_name) = &args.witness_name {
            let witness_path = save_witness_to_dir(witness_stack, witness_name, target_dir)?;

            println!("[{}] Witness saved to {}", package.name, witness_path.display());
        }
    }
    Ok(())
}

fn execute_program_and_decode(
    program: CompiledProgram,
    package: &Package,
    prover_name: &str,
    foreign_call_resolver_url: Option<&str>,
) -> Result<(Option<InputValue>, WitnessStack), CliError> {
    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(&package.root_dir, prover_name, Format::Toml, &program.abi)?;
    let witness_stack = execute_program(&program, &inputs_map, foreign_call_resolver_url)?;
    let public_abi = program.abi.public_abi();
    // Get the entry point witness for the ABI
    let main_witness =
        &witness_stack.peek().expect("Should have at least one witness on the stack").witness;
    let (_, return_value) = public_abi.decode(main_witness)?;

    Ok((return_value, witness_stack))
}

pub(crate) fn execute_program(
    compiled_program: &CompiledProgram,
    inputs_map: &InputMap,
    foreign_call_resolver_url: Option<&str>,
) -> Result<WitnessStack, CliError> {
    let blackbox_solver = Bn254BlackBoxSolver::new();

    let initial_witness = compiled_program.abi.encode(inputs_map, None)?;

    let solved_witness_stack_err = nargo::ops::execute_program(
        &compiled_program.program,
        initial_witness,
        &blackbox_solver,
        &mut DefaultForeignCallExecutor::new(true, foreign_call_resolver_url),
    );
    match solved_witness_stack_err {
        Ok(solved_witness_stack) => Ok(solved_witness_stack),
        Err(err) => {
            let debug_artifact = DebugArtifact {
                debug_symbols: compiled_program.debug.clone(),
                file_map: compiled_program.file_map.clone(),
                warnings: compiled_program.warnings.clone(),
            };

            if let Some(diagnostic) =
                try_to_diagnose_runtime_error(&err, &compiled_program.abi, &compiled_program.debug)
            {
                diagnostic.report(&debug_artifact, false);
            }

            Err(crate::errors::CliError::NargoError(err))
        }
    }
}
