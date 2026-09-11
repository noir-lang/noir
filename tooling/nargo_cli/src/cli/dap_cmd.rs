use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

use clap::Args;
use dap::events::OutputEventBody;
use dap::prelude::Event;
use dap::requests::Command;
use dap::responses::ResponseBody;
use dap::server::Server;
use dap::types::{Capabilities, OutputEventCategory};
use nargo::constants::PROVER_INPUT_FILE;
use nargo::foreign_calls::OracleResolverUrl;
use nargo::ops::debug::load_workspace_files;
use nargo::workspace::Workspace;
use nargo_toml::{PackageSelection, get_package_manifest, resolve_workspace_from_toml};
use noirc_driver::{CompileOptions, NOIR_ARTIFACT_VERSION_STRING};
use noirc_frontend::graph::CrateName;
use serde_json::Value;

use crate::cli::comptime_debugger::{ComptimeDapDebugger, SteppingMode};
use crate::cli::comptime_oracle::ComptimeForeignCallExecutor;
use crate::cli::debug_cmd::{DebugSession, prepare_debug_session};
use crate::errors::{CliError, DapError};

/// Command variants (with camelCase renaming) that are unit types and take no arguments.
/// Some DAP clients send `"arguments": {}` for these, which serde rejects.
const UNIT_COMMANDS: &[&str] = &["configurationDone", "loadedSources", "threads"];

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
    preflight_test_name: Option<String>,
}

/// What a DAP client asks us to debug, as sent in the `launch` request or the
/// preflight flags.
struct LaunchParams {
    project_folder: String,
    package: Option<String>,
    prover_name: String,
    test_name: Option<String>,
    oracle_resolver_url: Option<String>,
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

fn loop_uninitialized_dap<R: Read, W: Write>(mut server: Server<R, W>) -> Result<(), DapError> {
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

                // Clone all values from launch arguments into owned Strings
                // so we can release the borrow on `req` for `ack()`.
                let params = LaunchParams {
                    project_folder: project_folder.clone(),
                    package: additional_data
                        .get("package")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    prover_name: additional_data
                        .get("proverName")
                        .and_then(|v| v.as_str())
                        .unwrap_or(PROVER_INPUT_FILE)
                        .to_string(),
                    test_name: additional_data
                        .get("testName")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    oracle_resolver_url: additional_data
                        .get("oracleResolver")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                };

                eprintln!("Project folder: {}", params.project_folder);
                eprintln!("Package: {}", params.package.as_deref().unwrap_or("(default)"));
                eprintln!("Prover name: {}", params.prover_name);

                server.respond(req.ack()?)?;

                if let Err(e) = run_dap_loop(&mut server, &params) {
                    eprintln!("Debugger error: {e}");
                }
                break;
            }

            Command::Disconnect(_) => {
                server.respond(req.ack()?)?;
                break;
            }

            _ => {
                let command = req.command;
                eprintln!("ERROR: unhandled command in outer loop: {command:?}");
            }
        }
    }
    Ok(())
}

fn run_dap_loop<R: Read, W: Write>(
    server: &mut Server<R, W>,
    params: &LaunchParams,
) -> Result<(), DapError> {
    // Send Initialized immediately so VS Code shows the debug panel.
    // All errors after this point are reported via DAP Output events.
    server.send_event(Event::Initialized)?;

    let result = run_dap_loop_inner(server, params);

    if let Err(ref e) = result {
        send_error_to_dap(server, &format!("{e}"));
    }

    // Always send Terminated so VS Code knows the session is over.
    let _ = server.send_event(Event::Terminated(None));

    result
}

fn send_error_to_dap<R: Read, W: Write>(server: &mut Server<R, W>, message: &str) {
    let _ = server.send_event(Event::Output(OutputEventBody {
        category: Some(OutputEventCategory::Console),
        output: format!("Debugger error: {message}\n"),
        ..OutputEventBody::default()
    }));
}

fn run_dap_loop_inner<R: Read, W: Write>(
    server: &mut Server<R, W>,
    params: &LaunchParams,
) -> Result<(), DapError> {
    let workspace =
        find_workspace(&params.project_folder, params.package.as_deref()).ok_or_else(|| {
            DapError::Load(workspace_not_found_error_msg(
                &params.project_folder,
                params.package.as_deref(),
            ))
        })?;
    let package =
        workspace.into_iter().find(|p| p.is_binary() || p.is_contract()).ok_or_else(|| {
            DapError::Load("No matching binary or contract packages found in workspace".into())
        })?;

    let (file_manager, parsed_files) = load_workspace_files(&workspace);
    let DebugSession { mut context, func_id, func_args, test: _ } = prepare_debug_session(
        &file_manager,
        &parsed_files,
        &workspace,
        package,
        &CompileOptions::default(),
        &params.prover_name,
        params.test_name.as_deref(),
    )
    .map_err(|e| DapError::Load(e.to_string()))?;

    let oracle_resolver: Option<OracleResolverUrl> = params
        .oracle_resolver_url
        .as_deref()
        .map(str::parse)
        .transpose()
        .map_err(|e| DapError::Load(format!("Invalid oracle resolver URL: {e}")))?;
    let oracle_executor = ComptimeForeignCallExecutor::new(
        oracle_resolver.as_ref(),
        Some(workspace.root_dir.clone()),
        Some(package.name.to_string()),
    );

    // Run interpreter with debugger.
    // The debugger stops at the first statement (StepIn mode) and enters a DAP sub-loop
    // that handles all requests: SetBreakpoints, ConfigurationDone, StackTrace, Variables, etc.
    let breakpoints = HashMap::new();
    let debugger = ComptimeDapDebugger::new(server, breakpoints, SteppingMode::StepIn);

    let result = context.interpret_function_with_debugger(
        func_id,
        func_args,
        Box::new(debugger),
        Some(Box::new(oracle_executor)),
    );

    if let Err(err) = result {
        eprintln!("Interpreter error: {err:?}");
    }

    Ok(())
}

fn run_preflight_check(args: DapCommand) -> Result<(), DapError> {
    let Some(project_folder) = args.preflight_project_folder else {
        return Err(DapError::PreFlight("Noir Debugger could not initialize because the IDE (for example, VS Code) did not specify a project folder to debug.".into()));
    };

    let package = args.preflight_package.as_deref();
    let prover_name = args.preflight_prover_name.as_deref().unwrap_or(PROVER_INPUT_FILE);

    let workspace = find_workspace(&project_folder, package)
        .ok_or_else(|| DapError::Load(workspace_not_found_error_msg(&project_folder, package)))?;
    let package =
        workspace.into_iter().find(|p| p.is_binary() || p.is_contract()).ok_or_else(|| {
            DapError::Load(
                "No matching binary or contract packages found in workspace. Only these packages can be debugged.".into(),
            )
        })?;

    let (file_manager, parsed_files) = load_workspace_files(&workspace);
    prepare_debug_session(
        &file_manager,
        &parsed_files,
        &workspace,
        package,
        &CompileOptions::default(),
        prover_name,
        args.preflight_test_name.as_deref(),
    )
    .map_err(|e| DapError::Load(e.to_string()))?;

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
        return run_preflight_check(args).map_err(CliError::DapError);
    }

    let output = BufWriter::new(std::io::stdout());
    let input = BufReader::new(DapFixingReader::new(BufReader::new(std::io::stdin())));
    let server = Server::new(input, output);

    loop_uninitialized_dap(server).map_err(CliError::DapError)
}

/// Wraps a buffered reader over the DAP input stream and transparently fixes malformed messages
/// before they reach the [`Server`].
///
/// Specifically, some clients send `"arguments": {}` for unit-variant commands like
/// `configurationDone`. The `dap` crate's serde deserialization rejects non-null `arguments`
/// for unit variants. This reader strips empty `arguments` objects from those commands.
///
/// On any parsing failure the original bytes are passed through unchanged so that [`Server`]
/// can produce its own error.
struct DapFixingReader<R: BufRead> {
    inner: R,
    /// Pre-processed bytes of the next DAP message ready to be returned by `read`.
    pending: Vec<u8>,
    /// The number of bytes we have already copied to the destination in `read`.
    pos: usize,
}

impl<R: BufRead> DapFixingReader<R> {
    fn new(inner: R) -> Self {
        Self { inner, pending: Vec::new(), pos: 0 }
    }

    /// Read one DAP message from `inner`, fix it, and store it in `pending`.
    /// Returns `false` on EOF, `true` if a message was buffered.
    fn fill_pending(&mut self) -> std::io::Result<bool> {
        // Read the Content-Length header line.
        let mut line = String::new();
        if self.inner.read_line(&mut line)? == 0 {
            return Ok(false); // EOF
        }

        // Try to parse the content length.
        let content_length = line
            .trim_end()
            .strip_prefix("Content-Length:")
            .and_then(|rest| rest.trim().parse::<usize>().ok());

        let Some(content_length) = content_length else {
            // Not a valid Content-Length header; pass through and let the Server error.
            self.pending = line.into_bytes();
            self.pos = 0;
            return Ok(true);
        };

        // Read the blank separator line.
        // (In `Server::poll_request` this happens as part of the `loop`).
        let mut sep = String::new();
        if self.inner.read_line(&mut sep)? == 0 {
            return Ok(false); // EOF
        }

        // Read exactly content_length bytes.
        let mut content = vec![0u8; content_length];
        self.inner.read_exact(&mut content)?;

        // Fix the content, falling back to the original on any error.
        let fixed = fix_dap_content(&content);

        // Reconstruct the DAP framing with the (possibly updated) length.
        let header = format!("Content-Length: {}\r\n\r\n", fixed.len());
        self.pending = header.into_bytes();
        self.pending.extend_from_slice(&fixed);
        self.pos = 0;
        Ok(true)
    }
}

impl<R: BufRead> Read for DapFixingReader<R> {
    /// Read data from `pending` into `buf`
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // Refill if we've consumed everything in `pending`.
        while self.pos >= self.pending.len() {
            if !self.fill_pending()? {
                return Ok(0); // EOF
            }
        }
        let available = &self.pending[self.pos..];
        // How much data can we read depends on the size of `buf` and `available`.
        let n = buf.len().min(available.len());
        buf[..n].copy_from_slice(&available[..n]);
        self.pos += n;
        Ok(n)
    }
}

/// Remove an empty `arguments` object from JSON for unit-variant DAP commands.
///
/// Some clients send `{"command": "configurationDone", "arguments": {}}`, but the `dap` crate
/// expects no `arguments` key at all for unit variants.  Returns the original bytes unchanged if
/// parsing fails or no fix is needed.
fn fix_dap_content(content: &[u8]) -> Vec<u8> {
    let Ok(mut value) = serde_json::from_slice::<Value>(content) else {
        return content.to_vec();
    };

    let Some(obj) = value.as_object_mut() else {
        return content.to_vec();
    };

    let is_unit_command =
        obj.get("command").and_then(|v| v.as_str()).is_some_and(|cmd| UNIT_COMMANDS.contains(&cmd));

    if is_unit_command
        && let Some(Value::Object(args)) = obj.get("arguments")
        && args.is_empty()
    {
        obj.remove("arguments");
    }

    serde_json::to_vec(&value).unwrap_or_else(|_| content.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dap::requests::Request;

    fn make_dap_message(body: &str) -> String {
        format!("Content-Length: {}\r\n\r\n{}", body.len(), body)
    }

    fn read_all<R: BufRead>(reader: &mut DapFixingReader<R>) -> Vec<u8> {
        let mut out = Vec::new();
        Read::read_to_end(reader, &mut out).unwrap();
        out
    }

    /// Find the body after the separator line.
    fn find_body(output: &str) -> &str {
        let body_start = output.find("\r\n\r\n").unwrap() + 4;
        &output[body_start..]
    }

    #[test]
    fn test_empty_args() {
        let input = r#"{"seq":1,"command":"configurationDone","arguments":{}}"#;
        let _ = serde_json::from_str::<Request>(input).expect_err("empty args do not parse");
    }

    #[test]
    fn test_strips_empty_arguments_for_unit_commands() {
        let input = make_dap_message(r#"{"seq":1,"command":"configurationDone","arguments":{}}"#);
        let mut reader = DapFixingReader::new(BufReader::new(input.as_bytes()));
        let output = String::from_utf8(read_all(&mut reader)).unwrap();

        // The fixed body should not contain "arguments".
        let body: Value = serde_json::from_str(find_body(&output)).unwrap();
        assert!(body.get("arguments").is_none(), "arguments should be stripped");
        assert_eq!(body["command"], "configurationDone");

        let _ = serde_json::from_value::<Request>(body).expect("should parse request");
    }

    #[test]
    fn test_preserves_non_empty_arguments() {
        let input =
            make_dap_message(r#"{"seq":1,"command":"configurationDone","arguments":{"extra":1}}"#);
        let mut reader = DapFixingReader::new(BufReader::new(input.as_bytes()));
        let output = String::from_utf8(read_all(&mut reader)).unwrap();

        let body: Value = serde_json::from_str(find_body(&output)).unwrap();
        assert!(body.get("arguments").is_some(), "non-empty arguments should be preserved");

        let _ = serde_json::from_value::<Request>(body).expect_err("extra args do not parse");
    }

    #[test]
    fn test_passes_through_non_unit_commands_unchanged() {
        let json = r#"{"seq":1,"command":"initialize","arguments":{"adapterID":"test"}}"#;
        let input = make_dap_message(json);
        let mut reader = DapFixingReader::new(BufReader::new(input.as_bytes()));
        let output = String::from_utf8(read_all(&mut reader)).unwrap();

        let body: Value = serde_json::from_str(find_body(&output)).unwrap();
        assert!(body.get("arguments").is_some());

        let _ = serde_json::from_value::<Request>(body).expect("non unit request parses");
    }

    #[test]
    fn test_passes_through_invalid_json_unchanged() {
        let bad_json = "not json at all";
        let input = make_dap_message(bad_json);
        let mut reader = DapFixingReader::new(BufReader::new(input.as_bytes()));
        let output = String::from_utf8(read_all(&mut reader)).unwrap();

        assert_eq!(find_body(&output), bad_json);
    }

    #[test]
    fn test_passes_through_invalid_header_unchanged() {
        let input = "Content-Type: application/json\r\n\r\n[]";
        let mut reader = DapFixingReader::new(BufReader::new(input.as_bytes()));
        let output = String::from_utf8(read_all(&mut reader)).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn test_read_header_then_eof() {
        let input = "Content-Length: 10\r\n";
        let mut reader = DapFixingReader::new(BufReader::new(input.as_bytes()));
        let output = String::from_utf8(read_all(&mut reader)).unwrap();
        assert_eq!(output, "");
    }

    #[test]
    fn test_multiple_messages_in_sequence() {
        let msg1 = make_dap_message(r#"{"seq":1,"command":"configurationDone","arguments":{}}"#);
        let msg2 = make_dap_message(r#"{"seq":2,"command":"threads","arguments":{}}"#);
        let input = format!("{msg1}{msg2}");
        let mut reader = DapFixingReader::new(BufReader::new(input.as_bytes()));

        let mut buf = Vec::new();
        Read::read_to_end(&mut reader, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Both messages should have arguments stripped.
        assert_eq!(output.matches("\"arguments\"").count(), 0);
        assert_eq!(output.matches("configurationDone").count(), 1);
        assert_eq!(output.matches("threads").count(), 1);
    }
}
