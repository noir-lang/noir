use std::future::{self, Future};

use async_lsp::{ErrorCode, ResponseError};

use nargo::{package::Package, workspace::Workspace};
use noirc_driver::check_crate;
use noirc_frontend::hir::FunctionNameMatch;

use crate::{
    byte_span_to_range, prepare_source, resolve_workspace_for_source_path,
    types::{CodeLens, CodeLensParams, CodeLensResult, Command},
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
const DEBUG_COMMAND: &str = "nargo.debug.dap";
const DEBUG_CODELENS_TITLE: &str = "Debug";

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

    if let Some(collected_lenses) = state.cached_lenses.get(&params.text_document.uri.to_string()) {
        return Ok(Some(collected_lenses.clone()));
    }

    let source_string = std::fs::read_to_string(&file_path).map_err(|_| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "Could not read file from disk")
    })?;

    let workspace = resolve_workspace_for_source_path(file_path.as_path()).unwrap();

    let package = crate::workspace_package_for_file(&workspace, &file_path).ok_or_else(|| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "Could not find package for file")
    })?;

    let (mut context, crate_id) = prepare_source(source_string, state);
    // We ignore the warnings and errors produced by compilation for producing code lenses
    // because we can still get the test functions even if compilation fails
    let _ = check_crate(&mut context, crate_id, &Default::default());

    let collected_lenses =
        collect_lenses_for_package(&context, crate_id, &workspace, package, None);

    if collected_lenses.is_empty() {
        state.cached_lenses.remove(&params.text_document.uri.to_string());
        Ok(None)
    } else {
        state
            .cached_lenses
            .insert(params.text_document.uri.to_string().clone(), collected_lenses.clone());
        Ok(Some(collected_lenses))
    }
}

pub(crate) fn collect_lenses_for_package(
    context: &noirc_frontend::hir::Context,
    crate_id: noirc_frontend::graph::CrateId,
    workspace: &Workspace,
    package: &Package,
    file_path: Option<&std::path::PathBuf>,
) -> Vec<CodeLens> {
    let mut lenses: Vec<CodeLens> = vec![];
    let fm = &context.file_manager;
    let files = fm.as_file_map();
    let tests =
        context.get_all_test_functions_in_crate_matching(&crate_id, FunctionNameMatch::Anything);
    for (func_name, test_function) in tests {
        let location = context.function_meta(&test_function.get_id()).name.location;
        let file_id = location.file;

        // Ignore diagnostics for any file that wasn't the file we saved
        // TODO: In the future, we could create "related" diagnostics for these files
        if let Some(file_path) = file_path {
            if fm.path(file_id).expect("file must exist to contain tests") != *file_path {
                continue;
            }
        }

        let range = byte_span_to_range(files, file_id, location.span.into()).unwrap_or_default();

        let test_command = Command {
            title: with_arrow(TEST_CODELENS_TITLE),
            command: TEST_COMMAND.into(),
            arguments: Some(
                [
                    package_selection_args(workspace, package),
                    vec!["--exact".into(), "--show-output".into(), func_name.into()],
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
            if let Some(file_path) = file_path {
                if fm.path(file_id).expect("file must exist to contain `main` function")
                    != *file_path
                {
                    return lenses;
                }
            }

            let range =
                byte_span_to_range(files, file_id, location.span.into()).unwrap_or_default();

            let compile_command = Command {
                title: with_arrow(COMPILE_CODELENS_TITLE),
                command: COMPILE_COMMAND.into(),
                arguments: Some(package_selection_args(workspace, package)),
            };

            let compile_lens = CodeLens { range, command: Some(compile_command), data: None };

            lenses.push(compile_lens);

            let internal_command_lenses = [
                (INFO_CODELENS_TITLE, INFO_COMMAND),
                (EXECUTE_CODELENS_TITLE, EXECUTE_COMMAND),
                (DEBUG_CODELENS_TITLE, DEBUG_COMMAND),
            ]
            .map(|(title, command)| {
                let command = Command {
                    title: title.to_string(),
                    command: command.into(),
                    arguments: Some(package_selection_args(workspace, package)),
                };
                CodeLens { range, command: Some(command), data: None }
            });

            lenses.append(&mut Vec::from(internal_command_lenses));
        }
    }

    if package.is_contract() {
        // Currently not looking to deduplicate this since we don't have a clear decision on if the Contract stuff is staying
        for contract in context.get_all_contracts(&crate_id) {
            let location = contract.location;
            let file_id = location.file;

            // Ignore diagnostics for any file that wasn't the file we saved
            // TODO: In the future, we could create "related" diagnostics for these files
            if let Some(file_path) = file_path {
                if fm.path(file_id).expect("file must exist to contain a contract") != *file_path {
                    continue;
                }
            }

            let range =
                byte_span_to_range(files, file_id, location.span.into()).unwrap_or_default();

            let compile_command = Command {
                title: with_arrow(COMPILE_CODELENS_TITLE),
                command: COMPILE_COMMAND.into(),
                arguments: Some(package_selection_args(workspace, package)),
            };

            let compile_lens = CodeLens { range, command: Some(compile_command), data: None };

            lenses.push(compile_lens);

            let info_command = Command {
                title: INFO_CODELENS_TITLE.to_string(),
                command: INFO_COMMAND.into(),
                arguments: Some(package_selection_args(workspace, package)),
            };

            let info_lens = CodeLens { range, command: Some(info_command), data: None };

            lenses.push(info_lens);
        }
    }

    lenses
}
