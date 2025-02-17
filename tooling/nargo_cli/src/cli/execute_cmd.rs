use std::path::PathBuf;

use acvm::acir::native_types::WitnessStack;
use acvm::FieldElement;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;

use nargo::constants::PROVER_INPUT_FILE;
use nargo::errors::try_to_diagnose_runtime_error;
use nargo::foreign_calls::DefaultForeignCallBuilder;
use nargo::package::Package;
use nargo::workspace::Workspace;
use nargo::PrintOutput;
use nargo_toml::PackageSelection;
use noir_artifact_cli::fs::artifact::read_program_from_file;
use noir_artifact_cli::fs::inputs::read_inputs_from_file;
use noir_artifact_cli::fs::witness::save_witness_to_dir;
use noirc_abi::input_parser::InputValue;
use noirc_abi::InputMap;
use noirc_artifacts::debug::DebugArtifact;
use noirc_driver::{CompileOptions, CompiledProgram};

use super::compile_cmd::compile_workspace_full;
use super::{LockType, PackageOptions, WorkspaceCommand};
use crate::errors::CliError;

/// Executes a circuit to calculate its return value
#[derive(Debug, Clone, Args)]
#[clap(visible_alias = "e")]
pub(crate) struct ExecuteCommand {
    /// Write the execution witness to named file
    ///
    /// Defaults to the name of the package being executed.
    witness_name: Option<String>,

    /// The name of the toml file which contains the inputs for the prover
    #[clap(long, short, default_value = PROVER_INPUT_FILE)]
    prover_name: String,

    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// JSON RPC url to solve oracle calls
    #[clap(long)]
    oracle_resolver: Option<String>,
}

impl WorkspaceCommand for ExecuteCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }

    fn lock_type(&self) -> LockType {
        // Compiles artifacts.
        LockType::Exclusive
    }
}

pub(crate) fn run(args: ExecuteCommand, workspace: Workspace) -> Result<(), CliError> {
    let target_dir = &workspace.target_directory_path();

    // Compile the full workspace in order to generate any build artifacts.
    compile_workspace_full(&workspace, &args.compile_options)?;

    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());
    for package in binary_packages {
        let program_artifact_path = workspace.package_build_path(package);
        let program: CompiledProgram = read_program_from_file(&program_artifact_path)?.into();
        let abi = program.abi.clone();

        let results = execute_program_and_decode(
            program,
            package,
            &args.prover_name,
            args.oracle_resolver.as_deref(),
            Some(workspace.root_dir.clone()),
            Some(package.name.to_string()),
            args.compile_options.pedantic_solving,
        )?;

        println!("[{}] Circuit witness successfully solved", package.name);
        if let Some(ref return_value) = results.actual_return {
            println!("[{}] Circuit output: {return_value:?}", package.name);
        }

        let package_name = package.name.clone().into();
        let witness_name = args.witness_name.as_ref().unwrap_or(&package_name);
        let witness_path = save_witness_to_dir(&results.witness_stack, witness_name, target_dir)?;
        println!("[{}] Witness saved to {}", package.name, witness_path.display());

        // Sanity checks on the return value after the witness has been saved, so it can be inspected if necessary.
        if let Some(expected) = results.expected_return {
            if results.actual_return.as_ref() != Some(&expected) {
                return Err(CliError::UnexpectedReturn { expected, actual: results.actual_return });
            }
        }
        // We can expect that if the circuit returns something, it should be non-empty after execution.
        if let Some(ref expected) = abi.return_type {
            if results.actual_return.is_none() {
                return Err(CliError::MissingReturn { expected: expected.clone() });
            }
        }
    }
    Ok(())
}

fn execute_program_and_decode(
    program: CompiledProgram,
    package: &Package,
    prover_name: &str,
    foreign_call_resolver_url: Option<&str>,
    root_path: Option<PathBuf>,
    package_name: Option<String>,
    pedantic_solving: bool,
) -> Result<ExecutionResults, CliError> {
    // Parse the initial witness values from Prover.toml
    let (inputs_map, expected_return) = read_inputs_from_file(
        &package.root_dir.join(prover_name).with_extension("toml"),
        &program.abi,
    )?;
    let witness_stack = execute_program(
        &program,
        &inputs_map,
        foreign_call_resolver_url,
        root_path,
        package_name,
        pedantic_solving,
    )?;
    // Get the entry point witness for the ABI
    let main_witness =
        &witness_stack.peek().expect("Should have at least one witness on the stack").witness;
    let (_, actual_return) = program.abi.decode(main_witness)?;

    Ok(ExecutionResults { expected_return, actual_return, witness_stack })
}

struct ExecutionResults {
    expected_return: Option<InputValue>,
    actual_return: Option<InputValue>,
    witness_stack: WitnessStack<FieldElement>,
}

pub(crate) fn execute_program(
    compiled_program: &CompiledProgram,
    inputs_map: &InputMap,
    foreign_call_resolver_url: Option<&str>,
    root_path: Option<PathBuf>,
    package_name: Option<String>,
    pedantic_solving: bool,
) -> Result<WitnessStack<FieldElement>, CliError> {
    let initial_witness = compiled_program.abi.encode(inputs_map, None)?;

    let solved_witness_stack_err = nargo::ops::execute_program(
        &compiled_program.program,
        initial_witness,
        &Bn254BlackBoxSolver(pedantic_solving),
        &mut DefaultForeignCallBuilder {
            output: PrintOutput::Stdout,
            enable_mocks: false,
            resolver_url: foreign_call_resolver_url.map(|s| s.to_string()),
            root_path,
            package_name,
        }
        .build(),
    );
    match solved_witness_stack_err {
        Ok(solved_witness_stack) => Ok(solved_witness_stack),
        Err(err) => {
            let debug_artifact = DebugArtifact {
                debug_symbols: compiled_program.debug.clone(),
                file_map: compiled_program.file_map.clone(),
            };

            if let Some(diagnostic) =
                try_to_diagnose_runtime_error(&err, &compiled_program.abi, &compiled_program.debug)
            {
                diagnostic.report(&debug_artifact, false);
            }

            Err(CliError::NargoError(err))
        }
    }
}
