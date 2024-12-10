use std::path::PathBuf;

use acvm::acir::native_types::WitnessStack;
use acvm::FieldElement;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;

use fm::FileManager;
use nargo::constants::PROVER_INPUT_FILE;
use nargo::errors::CompileError;
use nargo::ops::{compile_program, compile_program_with_debug_instrumenter, report_errors};
use nargo::package::{CrateName, Package};
use nargo::workspace::Workspace;
use nargo::{insert_all_files_for_workspace_into_file_manager, parse_all};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_abi::input_parser::{Format, InputValue};
use noirc_abi::InputMap;
use noirc_driver::{
    file_manager_with_stdlib, CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING,
};
use noirc_frontend::debug::DebugInstrumenter;
use noirc_frontend::hir::ParsedFiles;

use super::compile_cmd::get_target_width;
use super::fs::{inputs::read_inputs_from_file, witness::save_witness_to_dir};
use super::NargoConfig;
use crate::errors::CliError;

/// Executes a circuit in debug mode
#[derive(Debug, Clone, Args)]
pub(crate) struct DebugCommand {
    /// Write the execution witness to named file
    witness_name: Option<String>,

    /// The name of the toml file which contains the inputs for the prover
    #[clap(long, short, default_value = PROVER_INPUT_FILE)]
    prover_name: String,

    /// The name of the package to execute
    #[clap(long)]
    pub(super) package: Option<CrateName>,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// Force ACIR output (disabling instrumentation)
    #[clap(long)]
    acir_mode: bool,

    /// Disable vars debug instrumentation (enabled by default)
    #[clap(long)]
    skip_instrumentation: Option<bool>,
}

pub(crate) fn run(args: DebugCommand, config: NargoConfig) -> Result<(), CliError> {
    let acir_mode = args.acir_mode;
    let skip_instrumentation = args.skip_instrumentation.unwrap_or(acir_mode);

    let toml_path = get_package_manifest(&config.program_dir)?;
    let selection = args.package.map_or(PackageSelection::DefaultOrAll, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;
    let target_dir = &workspace.target_directory_path();

    let Some(package) = workspace.into_iter().find(|p| p.is_binary()) else {
        println!(
            "No matching binary packages found in workspace. Only binary packages can be debugged."
        );
        return Ok(());
    };

    let compiled_program = compile_bin_package_for_debugging(
        &workspace,
        package,
        acir_mode,
        skip_instrumentation,
        args.compile_options.clone(),
    )?;

    let target_width =
        get_target_width(package.expression_width, args.compile_options.expression_width);

    let compiled_program = nargo::ops::transform_program(compiled_program, target_width);

    run_async(package, compiled_program, &args.prover_name, &args.witness_name, target_dir)
}

pub(crate) fn compile_bin_package_for_debugging(
    workspace: &Workspace,
    package: &Package,
    acir_mode: bool,
    skip_instrumentation: bool,
    compile_options: CompileOptions,
) -> Result<CompiledProgram, CompileError> {
    let mut workspace_file_manager = file_manager_with_stdlib(std::path::Path::new(""));
    insert_all_files_for_workspace_into_file_manager(workspace, &mut workspace_file_manager);
    let mut parsed_files = parse_all(&workspace_file_manager);

    let compile_options = CompileOptions {
        instrument_debug: !skip_instrumentation,
        force_brillig: !acir_mode,
        ..compile_options
    };

    let compilation_result = if !skip_instrumentation {
        let debug_state =
            instrument_package_files(&mut parsed_files, &workspace_file_manager, package);

        compile_program_with_debug_instrumenter(
            &workspace_file_manager,
            &parsed_files,
            workspace,
            package,
            &compile_options,
            None,
            debug_state,
        )
    } else {
        compile_program(
            &workspace_file_manager,
            &parsed_files,
            workspace,
            package,
            &compile_options,
            None,
        )
    };

    report_errors(
        compilation_result,
        &workspace_file_manager,
        compile_options.deny_warnings,
        compile_options.silence_warnings,
    )
}

/// Add debugging instrumentation to all parsed files belonging to the package
/// being compiled
fn instrument_package_files(
    parsed_files: &mut ParsedFiles,
    file_manager: &FileManager,
    package: &Package,
) -> DebugInstrumenter {
    // Start off at the entry path and read all files in the parent directory.
    let entry_path_parent = package
        .entry_path
        .parent()
        .unwrap_or_else(|| panic!("The entry path is expected to be a single file within a directory and so should have a parent {:?}", package.entry_path));

    let mut debug_instrumenter = DebugInstrumenter::default();

    for (file_id, parsed_file) in parsed_files.iter_mut() {
        let file_path =
            file_manager.path(*file_id).expect("Parsed file ID not found in file manager");
        for ancestor in file_path.ancestors() {
            if ancestor == entry_path_parent {
                // file is in package
                debug_instrumenter.instrument_module(&mut parsed_file.0);
            }
        }
    }

    debug_instrumenter
}

fn run_async(
    package: &Package,
    program: CompiledProgram,
    prover_name: &str,
    witness_name: &Option<String>,
    target_dir: &PathBuf,
) -> Result<(), CliError> {
    use tokio::runtime::Builder;
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();

    runtime.block_on(async {
        println!("[{}] Starting debugger", package.name);
        let (return_value, witness_stack) =
            debug_program_and_decode(program, package, prover_name)?;

        if let Some(solved_witness_stack) = witness_stack {
            println!("[{}] Circuit witness successfully solved", package.name);

            if let Some(return_value) = return_value {
                println!("[{}] Circuit output: {return_value:?}", package.name);
            }

            if let Some(witness_name) = witness_name {
                let witness_path =
                    save_witness_to_dir(solved_witness_stack, witness_name, target_dir)?;

                println!("[{}] Witness saved to {}", package.name, witness_path.display());
            }
        } else {
            println!("Debugger execution halted.");
        }

        Ok(())
    })
}

fn debug_program_and_decode(
    program: CompiledProgram,
    package: &Package,
    prover_name: &str,
) -> Result<(Option<InputValue>, Option<WitnessStack<FieldElement>>), CliError> {
    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(&package.root_dir, prover_name, Format::Toml, &program.abi)?;
    let program_abi = program.abi.clone();
    let witness_stack = debug_program(program, &inputs_map)?;

    match witness_stack {
        Some(witness_stack) => {
            let main_witness = &witness_stack
                .peek()
                .expect("Should have at least one witness on the stack")
                .witness;
            let (_, return_value) = program_abi.decode(main_witness)?;
            Ok((return_value, Some(witness_stack)))
        }
        None => Ok((None, None)),
    }
}

pub(crate) fn debug_program(
    compiled_program: CompiledProgram,
    inputs_map: &InputMap,
) -> Result<Option<WitnessStack<FieldElement>>, CliError> {
    let initial_witness = compiled_program.abi.encode(inputs_map, None)?;

    noir_debugger::run_repl_session(&Bn254BlackBoxSolver, compiled_program, initial_witness)
        .map_err(CliError::from)
}
