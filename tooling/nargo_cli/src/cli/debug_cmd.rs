use clap::Args;
use fm::FileManager;
use nargo::constants::PROVER_INPUT_FILE;
use nargo::foreign_calls::OracleResolverUrl;
use nargo::ops::debug::{TestDefinition, get_test_function_for_debug, load_workspace_files};
use nargo::ops::{
    TestStatus, check_crate_and_report_errors, test_status_comptime_interpret_result,
};
use nargo::package::{CrateName, Package};
use nargo::workspace::Workspace;
use nargo_toml::PackageSelection;
use noir_artifact_cli::fs::inputs::read_inputs_from_file;
use noirc_driver::CompileOptions;
use noirc_errors::Location;
use noirc_frontend::hir::comptime::Value;
use noirc_frontend::hir::{Context, ParsedFiles};
use noirc_frontend::node_interner::FuncId;

use super::{LockType, WorkspaceCommand};
use crate::cli::comptime_oracle::ComptimeForeignCallExecutor;
use crate::cli::comptime_repl_debugger::ComptimeReplDebugger;
use crate::cli::execute_cmd::interpret::input_values_to_comptime_values;
use crate::errors::CliError;

/// Executes a program in debug mode
#[derive(Debug, Clone, Args)]
pub(crate) struct DebugCommand {
    /// The name of the toml file which contains the inputs for the prover
    #[clap(long, short, default_value = PROVER_INPUT_FILE)]
    prover_name: String,

    /// The name of the package to execute
    #[clap(long)]
    package: Option<CrateName>,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// Name (or substring) of the test function to debug
    #[clap(long)]
    test_name: Option<String>,

    /// JSON RPC url to solve oracle calls
    #[clap(long)]
    oracle_resolver: Option<OracleResolverUrl>,
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
        // Reads the Prover.toml file but shouldn't conflict with others.
        LockType::None
    }
}

/// Everything the comptime interpreter needs to start debugging a function:
/// the type-checked context, the function to run and its arguments.
pub(crate) struct DebugSession<'a> {
    pub(crate) context: Context<'a, 'a>,
    pub(crate) func_id: FuncId,
    pub(crate) func_args: Vec<(Value, Location)>,
    /// Set when debugging a test function rather than `main`.
    pub(crate) test: Option<TestDefinition>,
}

/// Type-checks `package` and resolves the function to debug: the test matching
/// `test_name` when given, otherwise `main` with its arguments read from the
/// `prover_name` TOML file.
pub(crate) fn prepare_debug_session<'a>(
    file_manager: &'a FileManager,
    parsed_files: &'a ParsedFiles,
    workspace: &Workspace,
    package: &Package,
    compile_options: &CompileOptions,
    prover_name: &str,
    test_name: Option<&str>,
) -> Result<DebugSession<'a>, CliError> {
    let (mut context, crate_id) = nargo::prepare_package(file_manager, parsed_files, package);
    context.package_build_path = workspace.package_build_path(package);

    check_crate_and_report_errors(&mut context, crate_id, compile_options)?;

    if let Some(test_name) = test_name {
        let test = get_test_function_for_debug(crate_id, &context, test_name)
            .map_err(CliError::Generic)?;
        let func_id = test.function.id;
        return Ok(DebugSession { context, func_id, func_args: vec![], test: Some(test) });
    }

    let main_id = context
        .get_main_function(&crate_id)
        .ok_or_else(|| CliError::Generic("Could not find main function".to_string()))?;

    let func_meta = context.def_interner.function_meta(&main_id);
    let error_types = std::collections::BTreeMap::default();
    let abi = noirc_driver::gen_abi(&context, &main_id, func_meta.return_visibility, error_types);
    let (prover_input, _) =
        read_inputs_from_file(&package.root_dir.join(prover_name).with_extension("toml"), &abi)?;

    let func_args =
        input_values_to_comptime_values(&prover_input, func_meta, &context.def_interner);
    Ok(DebugSession { context, func_id: main_id, func_args, test: None })
}

pub(crate) fn run(args: DebugCommand, workspace: Workspace) -> Result<(), CliError> {
    let Some(package) = workspace.into_iter().find(|p| p.is_binary() || p.is_contract()) else {
        println!(
            "No matching binary or contract packages found in workspace. Only these packages can be debugged."
        );
        return Ok(());
    };

    let (file_manager, parsed_files) = load_workspace_files(&workspace);

    loop {
        let DebugSession { mut context, func_id, func_args, test } = prepare_debug_session(
            &file_manager,
            &parsed_files,
            &workspace,
            package,
            &args.compile_options,
            &args.prover_name,
            args.test_name.as_deref(),
        )?;

        println!("Debugger started. Type 'help' for commands.");

        let (debugger, restart_requested) = ComptimeReplDebugger::new();
        let oracle_executor = ComptimeForeignCallExecutor::new(
            args.oracle_resolver.as_ref(),
            Some(workspace.root_dir.clone()),
            Some(package.name.to_string()),
        );
        let result = context.interpret_function_with_debugger(
            func_id,
            func_args,
            Box::new(debugger),
            Some(Box::new(oracle_executor)),
        );

        if let Some(ref test) = test {
            let status = test_status_comptime_interpret_result(result, &test.function);
            let status_str = match &status {
                TestStatus::Pass => "ok",
                TestStatus::Skipped => "skipped",
                _ => "FAILED",
            };
            println!("[{}] Testing {} ... {}", package.name, test.name, status_str);
            match &status {
                TestStatus::Fail { message, .. } => eprintln!("{message}"),
                TestStatus::CompileError(diagnostic) => eprintln!("{}", diagnostic.message),
                _ => {}
            }
        } else {
            match result {
                Ok(value) => {
                    println!(
                        "Program completed. Return value: {}",
                        value.display(&context.def_interner, context.file_manager.as_file_map())
                    );
                }
                Err(err) => {
                    let diagnostic = noirc_errors::CustomDiagnostic::from(&err);
                    noirc_frontend::error_reporting::report_one(
                        &diagnostic,
                        &file_manager,
                        &parsed_files,
                        false,
                        false,
                    );
                }
            }
        }

        if restart_requested.get() {
            println!("Restarting debugger...");
            continue;
        }

        break;
    }

    Ok(())
}
