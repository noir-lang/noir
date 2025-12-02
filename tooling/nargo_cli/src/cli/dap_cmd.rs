use acvm::acir::circuit::ExpressionWidth;
use clap::Args;
use dap::errors::ServerError;
use dap::events::OutputEventBody;
use dap::requests::Command;
use dap::responses::ResponseBody;
use dap::server::Server;
use dap::types::{Capabilities, OutputEventCategory};
use nargo::constants::PROVER_INPUT_FILE;
use nargo::ops::debug::{
    TestDefinition, compile_bin_package_for_debugging, compile_options_for_debugging,
    compile_test_fn_for_debugging, get_test_function_for_debug, load_workspace_files,
    prepare_package_for_debug,
};
use nargo::ops::{TestStatus, check_crate_and_report_errors, test_status_program_compile_pass};
use nargo::package::Package;
use nargo::workspace::Workspace;
use nargo_toml::{PackageSelection, get_package_manifest, resolve_workspace_from_toml};
use noir_artifact_cli::fs::inputs::read_inputs_from_file;
use noir_debugger::{DebugExecutionResult, DebugProject, RunParams};
use noirc_abi::Abi;
use noirc_driver::{
    CompileOptions, CompiledProgram, DEFAULT_EXPRESSION_WIDTH, NOIR_ARTIFACT_VERSION_STRING,
};
use noirc_errors::debug_info::DebugInfo;
use noirc_frontend::graph::CrateName;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use serde_json::Value;

use crate::errors::CliError;

use noir_debugger::errors::{DapError, LoadError};

#[derive(Debug, Clone, Args)]
pub(crate) struct DapCommand {
    #[clap(long)]
    preflight_check: bool,

    #[clap(long)]
    preflight_project_folder: Option<String>,

    #[clap(long)]
    preflight_package: Option<String>,

    #[clap(long)]
    preflight_prover_name: Option<String>,

    #[clap(long)]
    preflight_generate_acir: bool,

    #[clap(long)]
    preflight_skip_instrumentation: bool,

    #[clap(long)]
    preflight_test_name: Option<String>,

    /// Use pedantic ACVM solving, i.e. double-check some black-box function
    /// assumptions when solving.
    /// This is disabled by default.
    #[arg(long, default_value = "false")]
    pedantic_solving: bool,
}

fn find_workspace(project_folder: &str, package: Option<&str>) -> Option<Workspace> {
    let Ok(toml_path) = get_package_manifest(Path::new(project_folder)) else {
        eprintln!("ERROR: Failed to get package manifest");
        return None;
    };
    let package = package.and_then(|p| serde_json::from_str::<CrateName>(p).ok());
    let selection = package.map_or(PackageSelection::DefaultOrAll, PackageSelection::Selected);
    match resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    ) {
        Ok(workspace) => Some(workspace),
        Err(err) => {
            eprintln!("ERROR: Failed to resolve workspace: {err}");
            None
        }
    }
}

fn workspace_not_found_error_msg(project_folder: &str, package: Option<&str>) -> String {
    match package {
        Some(pkg) => {
            format!(r#"Noir Debugger could not load program from {project_folder}, package {pkg}"#)
        }
        None => format!(r#"Noir Debugger could not load program from {project_folder}"#),
    }
}

fn compile_main(
    workspace: &Workspace,
    package: &Package,
    compile_options: &CompileOptions,
) -> Result<CompiledProgram, LoadError> {
    compile_bin_package_for_debugging(workspace, package, compile_options)
        .map_err(|_| LoadError::Generic("Failed to compile project".into()))
}

fn compile_test(
    workspace: &Workspace,
    package: &Package,
    compile_options: CompileOptions,
    test_name: String,
) -> Result<(CompiledProgram, TestDefinition), LoadError> {
    let (file_manager, mut parsed_files) = load_workspace_files(workspace);

    let (mut context, crate_id) =
        prepare_package_for_debug(&file_manager, &mut parsed_files, package, workspace);

    check_crate_and_report_errors(&mut context, crate_id, &compile_options)
        .map_err(|_| LoadError::Generic("Failed to compile project".into()))?;

    let test = get_test_function_for_debug(crate_id, &context, &test_name)
        .map_err(|_| LoadError::Generic("Failed to compile project".into()))?;

    let program = compile_test_fn_for_debugging(&test, &mut context, package, compile_options)
        .map_err(|_| LoadError::Generic("Failed to compile project".into()))?;
    Ok((program, test))
}

fn load_and_compile_project(
    project_folder: &str,
    package: Option<&str>,
    prover_name: &str,
    compile_options: CompileOptions,
    test_name: Option<String>,
) -> Result<(DebugProject, Option<TestDefinition>), LoadError> {
    let workspace = find_workspace(project_folder, package)
        .ok_or(LoadError::Generic(workspace_not_found_error_msg(project_folder, package)))?;
    let package = workspace
        .into_iter()
        .find(|p| p.is_binary() || p.is_contract())
        .ok_or(LoadError::Generic("No matching binary or contract packages found in workspace. Only these packages can be debugged.".into()))?;

    let (compiled_program, test_def) = match test_name {
        None => {
            let program = compile_main(&workspace, package, &compile_options)?;
            Ok((program, None))
        }
        Some(test_name) => {
            let (program, test_def) =
                compile_test(&workspace, package, compile_options, test_name)?;
            Ok((program, Some(test_def)))
        }
    }?;

    let (inputs_map, _) = read_inputs_from_file(
        &package.root_dir.join(prover_name).with_extension("toml"),
        &compiled_program.abi,
    )
    .map_err(|e| {
        LoadError::Generic(format!("Failed to read program inputs from {prover_name}: {e}"))
    })?;
    let initial_witness = compiled_program
        .abi
        .encode(&inputs_map, None)
        .map_err(|_| LoadError::Generic("Failed to encode inputs".into()))?;

    let project = DebugProject {
        compiled_program,
        initial_witness,
        root_dir: workspace.root_dir.clone(),
        package_name: package.name.to_string(),
    };
    Ok((project, test_def))
}

fn loop_uninitialized_dap<R: Read, W: Write>(
    mut server: Server<R, W>,
    expression_width: ExpressionWidth,
    pedantic_solving: bool,
) -> Result<(), DapError> {
    while let Some(req) = server.poll_request()? {
        match req.command {
            Command::Initialize(_) => {
                let rsp = req.success(ResponseBody::Initialize(Capabilities {
                    supports_disassemble_request: Some(true),
                    supports_instruction_breakpoints: Some(true),
                    supports_stepping_granularity: Some(true),
                    ..Default::default()
                }));
                server.respond(rsp)?;
            }

            Command::Launch(ref arguments) => {
                let Some(Value::Object(ref additional_data)) = arguments.additional_data else {
                    server.respond(req.error("Missing launch arguments"))?;
                    continue;
                };
                let Some(Value::String(project_folder)) = additional_data.get("projectFolder")
                else {
                    server.respond(req.error("Missing project folder argument"))?;
                    continue;
                };

                let project_folder = project_folder.as_str();
                let package = additional_data.get("package").and_then(|v| v.as_str());
                let prover_name = additional_data
                    .get("proverName")
                    .and_then(|v| v.as_str())
                    .unwrap_or(PROVER_INPUT_FILE);

                let generate_acir =
                    additional_data.get("generateAcir").and_then(|v| v.as_bool()).unwrap_or(false);
                let skip_instrumentation = additional_data
                    .get("skipInstrumentation")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(generate_acir);
                let test_name =
                    additional_data.get("testName").and_then(|v| v.as_str()).map(String::from);
                let oracle_resolver_url = additional_data
                    .get("oracleResolver")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                eprintln!("Project folder: {project_folder}");
                eprintln!("Package: {}", package.unwrap_or("(default)"));
                eprintln!("Prover name: {prover_name}");

                let compile_options = compile_options_for_debugging(
                    generate_acir,
                    skip_instrumentation,
                    Some(expression_width),
                    CompileOptions::default(),
                );

                match load_and_compile_project(
                    project_folder,
                    package,
                    prover_name,
                    compile_options,
                    test_name,
                ) {
                    Ok((project, test)) => {
                        server.respond(req.ack()?)?;
                        let abi = project.compiled_program.abi.clone();
                        let debug = project.compiled_program.debug.clone();

                        let result = noir_debugger::run_dap_loop(
                            &mut server,
                            project,
                            RunParams {
                                oracle_resolver_url,
                                pedantic_solving,
                                raw_source_printing: None,
                            },
                        )?;

                        if let Some(test) = test {
                            analyze_test_result(&mut server, result, test, abi, debug)?;
                        }
                        break;
                    }
                    Err(LoadError::Generic(message)) => {
                        server.respond(req.error(message.as_str()))?;
                    }
                }
            }

            Command::Disconnect(_) => {
                server.respond(req.ack()?)?;
                break;
            }

            _ => {
                let command = req.command;
                eprintln!("ERROR: unhandled command: {command:?}");
            }
        }
    }
    Ok(())
}

fn analyze_test_result<R: Read, W: Write>(
    server: &mut Server<R, W>,
    result: DebugExecutionResult,
    test: TestDefinition,
    abi: Abi,
    debug: Vec<DebugInfo>,
) -> Result<(), ServerError> {
    let test_status = match result {
        DebugExecutionResult::Solved(result) => {
            test_status_program_compile_pass(&test.function, &abi, &debug, &Ok(result))
        }
        // Test execution failed
        DebugExecutionResult::Error(error) => {
            test_status_program_compile_pass(&test.function, &abi, &debug, &Err(error))
        }
        // Execution didn't complete
        DebugExecutionResult::Incomplete => {
            TestStatus::Fail { message: "Execution halted".into(), error_diagnostic: None }
        }
    };

    let test_result_message = match test_status {
        TestStatus::Pass => "✓ Test passed".into(),
        TestStatus::Fail { message, error_diagnostic } => {
            let basic_message = format!("x Test failed: {message}");
            match error_diagnostic {
                Some(diagnostic) => format!("{basic_message}.\n{diagnostic:#?}"),
                None => basic_message,
            }
        }
        TestStatus::CompileError(diagnostic) => format!("x Test failed.\n{diagnostic:#?}"),
        TestStatus::Skipped => "* Test skipped".into(),
    };

    server.send_event(dap::events::Event::Output(OutputEventBody {
        category: Some(OutputEventCategory::Console),
        output: test_result_message,
        ..OutputEventBody::default()
    }))
}

fn run_preflight_check(
    expression_width: ExpressionWidth,
    args: DapCommand,
) -> Result<(), DapError> {
    let project_folder = if let Some(project_folder) = args.preflight_project_folder {
        project_folder
    } else {
        return Err(DapError::PreFlightGenericError("Noir Debugger could not initialize because the IDE (for example, VS Code) did not specify a project folder to debug.".into()));
    };

    let package = args.preflight_package.as_deref();
    let test_name = args.preflight_test_name;
    let prover_name = args.preflight_prover_name.as_deref().unwrap_or(PROVER_INPUT_FILE);

    let compile_options: CompileOptions = compile_options_for_debugging(
        args.preflight_generate_acir,
        args.preflight_skip_instrumentation,
        Some(expression_width),
        CompileOptions::default(),
    );

    let _ = load_and_compile_project(
        project_folder.as_str(),
        package,
        prover_name,
        compile_options,
        test_name,
    )?;

    Ok(())
}

pub(crate) fn run(args: DapCommand) -> Result<(), CliError> {
    // When the --preflight-check flag is present, we run Noir's DAP server in "pre-flight mode", which test runs
    // the DAP initialization code without actually starting the DAP server.
    //
    // This lets the client IDE present any initialization issues (compiler version mismatches, missing prover files, etc)
    // in its own interface.
    //
    // This was necessary due to the VS Code project being reluctant to let extension authors capture
    // stderr output generated by a DAP server wrapped in DebugAdapterExecutable.
    //
    // Exposing this preflight mode lets us gracefully handle errors that happen *before*
    // the DAP loop is established, which otherwise are considered "out of band" by the maintainers of the DAP spec.
    // More details here: https://github.com/microsoft/vscode/issues/108138
    if args.preflight_check {
        return run_preflight_check(DEFAULT_EXPRESSION_WIDTH, args).map_err(CliError::DapError);
    }

    let output = BufWriter::new(std::io::stdout());
    let input = BufReader::new(std::io::stdin());
    let server = Server::new(input, output);

    loop_uninitialized_dap(server, DEFAULT_EXPRESSION_WIDTH, args.pedantic_solving)
        .map_err(CliError::DapError)
}
