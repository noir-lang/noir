use backend_interface::Backend;
use clap::Args;
use nargo::constants::PROVER_INPUT_FILE;
use nargo::workspace::Workspace;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_abi::input_parser::Format;
use noirc_driver::{CompileOptions, NOIR_ARTIFACT_VERSION_STRING};
use noirc_frontend::graph::CrateName;

use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use dap::errors::ServerError;
use dap::requests::Command;
use dap::responses::ResponseBody;
use dap::server::Server;
use dap::types::Capabilities;
use serde_json::Value;

use super::compile_cmd::compile_bin_package;
use super::fs::inputs::read_inputs_from_file;
use crate::errors::CliError;

use super::NargoConfig;

#[derive(Debug, Clone, Args)]
pub(crate) struct DapCommand;

fn find_workspace(project_folder: &str, package: Option<&str>) -> Option<Workspace> {
    let Ok(toml_path) = get_package_manifest(Path::new(project_folder)) else {
        return None;
    };
    let package = package.and_then(|p| serde_json::from_str::<CrateName>(p).ok());
    let selection = package.map_or(PackageSelection::DefaultOrAll, PackageSelection::Selected);
    let Ok(workspace) = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    ) else {
        return None;
    };
    Some(workspace)
}

fn loop_uninitialized<R: Read, W: Write>(
    mut server: Server<R, W>,
    backend: &Backend,
) -> Result<(), ServerError> {
    loop {
        let req = match server.poll_request()? {
            Some(req) => req,
            None => break,
        };
        match req.command {
            Command::Initialize(_) => {
                let rsp =
                    req.success(ResponseBody::Initialize(Capabilities { ..Default::default() }));
                server.respond(rsp)?;
            }

            Command::Launch(ref arguments) => {
                if let Some(Value::Object(ref data)) = arguments.additional_data {
                    if let Some(Value::String(ref project_folder)) = data.get("projectFolder") {
                        let project_folder = project_folder.as_str();
                        let package = data.get("package").and_then(|v| v.as_str());
                        let prover_name = data
                            .get("proverName")
                            .and_then(|v| v.as_str())
                            .unwrap_or(PROVER_INPUT_FILE);

                        eprintln!("Project folder: {}", project_folder);
                        eprintln!("Package: {}", package.unwrap_or("(none)"));
                        eprintln!("Prover name: {}", prover_name);

                        let Some(workspace) = find_workspace(project_folder, package) else {
                            server.respond(req.error("Cannot open workspace"))?;
                            continue;
                        };
                        let Ok((np_language, opcode_support)) = backend.get_backend_info() else {
                            server.respond(req.error("Failed to get backend info"))?;
                            continue;
                        };

                        let Some(package) = workspace.into_iter().find(|p| p.is_binary()) else {
                            server.respond(req.error("No matching binary packages found in workspace"))?;
                            continue;
                        };
                        let Ok(compiled_program) = compile_bin_package(
                            &workspace,
                            package,
                            &CompileOptions::default(),
                            np_language,
                            &|opcode| opcode_support.is_opcode_supported(opcode),
                        ) else {
                            server.respond(req.error("Failed to compile project"))?;
                            continue;
                        };
                        let Ok((inputs_map, _)) = read_inputs_from_file(
                            &package.root_dir,
                            prover_name,
                            Format::Toml,
                            &compiled_program.abi,
                        ) else {
                            server.respond(req.error("Failed to read program inputs"))?;
                            continue;
                        };
                        let Ok(initial_witness) = compiled_program.abi.encode(&inputs_map, None) else {
                            server.respond(req.error("Failed to encode inputs"))?;
                            continue;
                        };
                        #[allow(deprecated)]
                        let blackbox_solver =
                            barretenberg_blackbox_solver::BarretenbergSolver::new();

                        server.respond(req.ack()?)?;
                        noir_debugger::loop_initialized(
                            server,
                            &blackbox_solver,
                            compiled_program,
                            initial_witness,
                        )?;
                        break;
                    } else {
                        server.respond(req.error("Missing project folder argument"))?;
                    }
                } else {
                    server.respond(req.error("Missing launch arguments"))?;
                }
            }

            Command::Disconnect(_) => {
                server.respond(req.ack()?)?;
                break;
            }

            _ => {
                eprintln!("ERROR: unhandled command");
            }
        }
    }
    Ok(())
}

pub(crate) fn run(
    backend: &Backend,
    _args: DapCommand,
    _config: NargoConfig,
) -> Result<(), CliError> {
    // noir_debugger::start_dap_server(backend).map_err(CliError::DapError);
    let output = BufWriter::new(std::io::stdout());
    let input = BufReader::new(std::io::stdin());
    let server = Server::new(input, output);

    loop_uninitialized(server, backend).map_err(CliError::DapError)
}
