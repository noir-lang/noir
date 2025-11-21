use std::path::Path;
use std::time::Duration;

use acvm::FieldElement;
use acvm::acir::native_types::{WitnessMap, WitnessStack};
use clap::Args;
use fm::FileManager;
use nargo::constants::PROVER_INPUT_FILE;
use nargo::ops::debug::{
    TestDefinition, compile_bin_package_for_debugging, compile_options_for_debugging,
    compile_test_fn_for_debugging, get_test_function_for_debug, load_workspace_files,
    prepare_package_for_debug,
};
use nargo::ops::{
    TestStatus, check_crate_and_report_errors, test_status_program_compile_fail,
    test_status_program_compile_pass,
};
use nargo::package::{CrateName, Package};
use nargo::workspace::Workspace;
use nargo_toml::PackageSelection;
use noir_artifact_cli::execution::input_value_to_string;
use noir_artifact_cli::fs::inputs::read_inputs_from_file;
use noir_artifact_cli::fs::witness::save_witness_to_dir;
use noir_debugger::{DebugExecutionResult, DebugProject, RunParams};
use noirc_abi::Abi;
use noirc_driver::{CompileOptions, CompiledProgram};
use noirc_frontend::hir::Context;

use super::test_cmd::TestResult;
use super::test_cmd::formatters::Formatter;
use super::{LockType, WorkspaceCommand};
use crate::cli::test_cmd::formatters::PrettyFormatter;
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
    package: Option<CrateName>,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// Force ACIR output (disabling instrumentation)
    #[clap(long)]
    acir_mode: bool,

    /// Disable vars debug instrumentation (enabled by default)
    #[clap(long)]
    skip_instrumentation: Option<bool>,

    /// Raw string printing of source for testing
    #[clap(long, hide = true)]
    raw_source_printing: Option<bool>,

    /// Name (or substring) of the test function to debug
    #[clap(long)]
    test_name: Option<String>,

    /// JSON RPC url to solve oracle calls
    #[clap(long)]
    oracle_resolver: Option<String>,
}

// TODO: find a better name
struct PackageParams<'a> {
    prover_name: String,
    witness_name: Option<String>,
    target_dir: &'a Path,
}

impl WorkspaceCommand for DebugCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package
            .as_ref()
            .cloned()
            .map_or(PackageSelection::DefaultOrAll, PackageSelection::Selected)
    }

    fn lock_type(&self) -> LockType {
        // Always compiles fresh in-memory in debug mode, doesn't read or write the compilation artifacts.
        // Reads the Prover.toml file and writes the witness at the end, but shouldn't conflict with others.
        LockType::None
    }
}

pub(crate) fn run(args: DebugCommand, workspace: Workspace) -> Result<(), CliError> {
    let acir_mode = args.acir_mode;
    let skip_instrumentation = args.skip_instrumentation.unwrap_or(acir_mode);

    let package_params = PackageParams {
        prover_name: args.prover_name,
        witness_name: args.witness_name,
        target_dir: &workspace.target_directory_path(),
    };
    let run_params = RunParams {
        pedantic_solving: args.compile_options.pedantic_solving,
        raw_source_printing: args.raw_source_printing,
        oracle_resolver_url: args.oracle_resolver,
    };
    let workspace_clone = workspace.clone();

    let Some(package) = workspace_clone.into_iter().find(|p| p.is_binary() || p.is_contract())
    else {
        println!(
            "No matching binary or contract packages found in workspace. Only these packages can be debugged."
        );
        return Ok(());
    };

    let compile_options =
        compile_options_for_debugging(acir_mode, skip_instrumentation, None, args.compile_options);

    if let Some(test_name) = args.test_name {
        debug_test(test_name, package, workspace, compile_options, run_params, package_params)
    } else {
        debug_main(package, workspace, compile_options, run_params, package_params)
    }
}

fn print_test_result(test_result: TestResult, file_manager: &FileManager) {
    let formatter: Box<dyn Formatter> = Box::new(PrettyFormatter);
    formatter
        .test_end_sync(&test_result, 1, 1, file_manager, true, false, false)
        .expect("Could not display test result");
}

fn debug_test_fn(
    test: &TestDefinition,
    context: &mut Context,
    workspace: &Workspace,
    package: &Package,
    compile_options: CompileOptions,
    run_params: RunParams,
    package_params: PackageParams,
) -> TestResult {
    let compiled_program = compile_test_fn_for_debugging(test, context, compile_options);

    let test_status = match compiled_program {
        Ok(compiled_program) => {
            let abi = compiled_program.abi.clone();
            let debug = compiled_program.debug.clone();

            // Run debugger
            let debug_result =
                run_async(package, compiled_program, workspace, run_params, package_params);

            match debug_result {
                Ok(DebugExecutionResult::Solved(result)) => {
                    test_status_program_compile_pass(&test.function, &abi, &debug, &Ok(result))
                }
                Ok(DebugExecutionResult::Error(error)) => {
                    test_status_program_compile_pass(&test.function, &abi, &debug, &Err(error))
                }
                Ok(DebugExecutionResult::Incomplete) => TestStatus::Fail {
                    message: "Incomplete execution. Debugger halted".to_string(),
                    error_diagnostic: None,
                },
                Err(error) => TestStatus::Fail {
                    message: format!("Debugger failed: {error}"),
                    error_diagnostic: None,
                },
            }
        }
        Err(err) => test_status_program_compile_fail(err, &test.function),
    };

    TestResult::new(
        test.name.clone(),
        package.name.to_string(),
        test_status,
        String::new(),
        Duration::from_secs(1), // FIXME: hardcoded value
    )
}

fn debug_main(
    package: &Package,
    workspace: Workspace,
    compile_options: CompileOptions,
    run_params: RunParams,
    package_params: PackageParams,
) -> Result<(), CliError> {
    let compiled_program =
        compile_bin_package_for_debugging(&workspace, package, &compile_options)?;

    run_async(package, compiled_program, &workspace, run_params, package_params)?;

    Ok(())
}

fn debug_test(
    test_name: String,
    package: &Package,
    workspace: Workspace,
    compile_options: CompileOptions,
    run_params: RunParams,
    package_params: PackageParams,
) -> Result<(), CliError> {
    let (file_manager, mut parsed_files) = load_workspace_files(&workspace);

    let (mut context, crate_id) =
        prepare_package_for_debug(&file_manager, &mut parsed_files, package, &workspace);

    check_crate_and_report_errors(&mut context, crate_id, &compile_options)?;

    let test =
        get_test_function_for_debug(crate_id, &context, &test_name).map_err(CliError::Generic)?;

    let test_result = debug_test_fn(
        &test,
        &mut context,
        &workspace,
        package,
        compile_options,
        run_params,
        package_params,
    );
    print_test_result(test_result, &file_manager);

    Ok(())
}

fn run_async(
    package: &Package,
    program: CompiledProgram,
    workspace: &Workspace,
    run_params: RunParams,
    package_params: PackageParams,
) -> Result<DebugExecutionResult, CliError> {
    use tokio::runtime::Builder;
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();
    let abi = &program.abi.clone();

    runtime.block_on(async {
        println!("[{}] Starting debugger", package.name);
        let initial_witness = parse_initial_witness(package, &package_params.prover_name, abi)?;

        let project = DebugProject {
            compiled_program: program,
            initial_witness,
            root_dir: workspace.root_dir.clone(),
            package_name: package.name.to_string(),
        };
        let result = noir_debugger::run_repl_session(project, run_params);

        if let DebugExecutionResult::Solved(ref witness_stack) = result {
            println!("[{}] Circuit witness successfully solved", package.name);
            decode_and_save_program_witness(
                &package.name,
                witness_stack,
                abi,
                package_params.witness_name,
                package_params.target_dir,
            )?;
        }

        Ok(result)
    })
}

fn decode_and_save_program_witness(
    package_name: &CrateName,
    witness_stack: &WitnessStack<FieldElement>,
    abi: &Abi,
    target_witness_name: Option<String>,
    target_dir: &Path,
) -> Result<(), CliError> {
    let main_witness =
        &witness_stack.peek().expect("Should have at least one witness on the stack").witness;

    if let (_, Some(return_value)) = abi.decode(main_witness)? {
        let abi_type = &abi.return_type.as_ref().unwrap().abi_type;
        let output_string = input_value_to_string(&return_value, abi_type);
        println!("[{package_name}] Circuit output: {output_string}");
    }

    if let Some(witness_name) = target_witness_name {
        let mut witness_path = save_witness_to_dir(witness_stack, &witness_name, target_dir)?;

        // See if we can make the file path a bit shorter/easier to read if it starts with the current directory
        if let Ok(current_dir) = std::env::current_dir() {
            if let Ok(name_without_prefix) = witness_path.strip_prefix(current_dir) {
                witness_path = name_without_prefix.to_path_buf();
            }
        }
        println!("[{}] Witness saved to {}", package_name, witness_path.display());
    }
    Ok(())
}

fn parse_initial_witness(
    package: &Package,
    prover_name: &str,
    abi: &Abi,
) -> Result<WitnessMap<FieldElement>, CliError> {
    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(&package.root_dir.join(prover_name).with_extension("toml"), abi)?;
    let initial_witness = abi.encode(&inputs_map, None)?;
    Ok(initial_witness)
}
