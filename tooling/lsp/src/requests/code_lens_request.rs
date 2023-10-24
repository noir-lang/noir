use std::future::{self, Future};

use async_lsp::{ErrorCode, LanguageClient, ResponseError};

use nargo::{package::Package, prepare_package, workspace::Workspace};
use nargo_toml::{find_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::check_crate;
use noirc_frontend::hir::FunctionNameMatch;

use crate::{
    byte_span_to_range, get_non_stdlib_asset,
    types::{CodeLens, CodeLensParams, CodeLensResult, Command, LogMessageParams, MessageType},
    LspState,
};

const ARROW: &str = "▶\u{fe0e}";
const TEST_COMMAND: &str = "nargo.test";
const TEST_CODELENS_TITLE: &str = "Run Test";
const COMPILE_COMMAND: &str = "nargo.compile";
const COMPILE_CODELENS_TITLE: &str = "Compile";
const INFO_COMMAND: &str = "nargo.info";
const INFO_CODELENS_TITLE: &str = "Info";
const EXECUTE_COMMAND: &str = "nargo.execute";
const EXECUTE_CODELENS_TITLE: &str = "Execute";

fn with_arrow(title: &str) -> String {
    format!("{ARROW} {title}")
}

fn package_selection_args(workspace: &Workspace, package: &Package) -> Vec<serde_json::Value> {
    vec![
        "--program-dir".into(),
        workspace.root_dir.display().to_string().into(),
        "--package".into(),
        package.name.to_string().into(),
    ]
}

pub(crate) fn on_code_lens_request(
    state: &mut LspState,
    params: CodeLensParams,
) -> impl Future<Output = Result<CodeLensResult, ResponseError>> {
    future::ready(on_code_lens_request_inner(state, params))
}

fn on_code_lens_request_inner(
    state: &mut LspState,
    params: CodeLensParams,
) -> Result<CodeLensResult, ResponseError> {
    let file_path = params.text_document.uri.to_file_path().map_err(|_| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "URI is not a valid file path")
    })?;

    let root_path = state.root_path.as_deref().ok_or_else(|| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "Could not find project root")
    })?;

    let toml_path = match find_package_manifest(root_path, &file_path) {
        Ok(toml_path) => toml_path,
        Err(err) => {
            // If we cannot find a manifest, we log a warning but return no code lenses
            // We can reconsider this when we can build a file without the need for a Nargo.toml file to resolve deps
            let _ = state.client.log_message(LogMessageParams {
                typ: MessageType::WARNING,
                message: err.to_string(),
            });
            return Ok(None);
        }
    };
    let workspace =
        resolve_workspace_from_toml(&toml_path, PackageSelection::All).map_err(|err| {
            // If we found a manifest, but the workspace is invalid, we raise an error about it
            ResponseError::new(ErrorCode::REQUEST_FAILED, err)
        })?;

    let mut lenses: Vec<CodeLens> = vec![];

    for package in &workspace {
        let (mut context, crate_id) = prepare_package(package, Box::new(get_non_stdlib_asset));
        // We ignore the warnings and errors produced by compilation for producing code lenses
        // because we can still get the test functions even if compilation fails
        let _ = check_crate(&mut context, crate_id, false);

        let fm = &context.file_manager;
        let files = fm.as_file_map();
        let tests = context
            .get_all_test_functions_in_crate_matching(&crate_id, FunctionNameMatch::Anything);

        for (func_name, test_function) in tests {
            let location = context.function_meta(&test_function.get_id()).name.location;
            let file_id = location.file;

            // Ignore diagnostics for any file that wasn't the file we saved
            // TODO: In the future, we could create "related" diagnostics for these files
            if fm.path(file_id) != file_path {
                continue;
            }

            let range =
                byte_span_to_range(files, file_id, location.span.into()).unwrap_or_default();

            let test_command = Command {
                title: with_arrow(TEST_CODELENS_TITLE),
                command: TEST_COMMAND.into(),
                arguments: Some(
                    [
                        package_selection_args(&workspace, package),
                        vec!["--exact".into(), func_name.into()],
                    ]
                    .concat(),
                ),
            };

            let test_lens = CodeLens { range, command: Some(test_command), data: None };

            lenses.push(test_lens);
        }

        if package.is_binary() {
            if let Some(main_func_id) = context.get_main_function(&crate_id) {
                let location = context.function_meta(&main_func_id).name.location;
                let file_id = location.file;

                // Ignore diagnostics for any file that wasn't the file we saved
                // TODO: In the future, we could create "related" diagnostics for these files
                if fm.path(file_id) != file_path {
                    continue;
                }

                let range =
                    byte_span_to_range(files, file_id, location.span.into()).unwrap_or_default();

                let compile_command = Command {
                    title: with_arrow(COMPILE_CODELENS_TITLE),
                    command: COMPILE_COMMAND.into(),
                    arguments: Some(package_selection_args(&workspace, package)),
                };

                let compile_lens = CodeLens { range, command: Some(compile_command), data: None };

                lenses.push(compile_lens);

                let info_command = Command {
                    title: INFO_CODELENS_TITLE.to_string(),
                    command: INFO_COMMAND.into(),
                    arguments: Some(package_selection_args(&workspace, package)),
                };

                let info_lens = CodeLens { range, command: Some(info_command), data: None };

                lenses.push(info_lens);

                let execute_command = Command {
                    title: EXECUTE_CODELENS_TITLE.to_string(),
                    command: EXECUTE_COMMAND.into(),
                    arguments: Some(package_selection_args(&workspace, package)),
                };

                let execute_lens = CodeLens { range, command: Some(execute_command), data: None };

                lenses.push(execute_lens);
            }
        }

        if package.is_contract() {
            // Currently not looking to deduplicate this since we don't have a clear decision on if the Contract stuff is staying
            for contract in context.get_all_contracts(&crate_id) {
                let location = contract.location;
                let file_id = location.file;

                // Ignore diagnostics for any file that wasn't the file we saved
                // TODO: In the future, we could create "related" diagnostics for these files
                if fm.path(file_id) != file_path {
                    continue;
                }

                let range =
                    byte_span_to_range(files, file_id, location.span.into()).unwrap_or_default();

                let compile_command = Command {
                    title: with_arrow(COMPILE_CODELENS_TITLE),
                    command: COMPILE_COMMAND.into(),
                    arguments: Some(package_selection_args(&workspace, package)),
                };

                let compile_lens = CodeLens { range, command: Some(compile_command), data: None };

                lenses.push(compile_lens);

                let info_command = Command {
                    title: INFO_CODELENS_TITLE.to_string(),
                    command: INFO_COMMAND.into(),
                    arguments: Some(package_selection_args(&workspace, package)),
                };

                let info_lens = CodeLens { range, command: Some(info_command), data: None };

                lenses.push(info_lens);
            }
        }
    }

    if lenses.is_empty() {
        Ok(None)
    } else {
        Ok(Some(lenses))
    }
}
