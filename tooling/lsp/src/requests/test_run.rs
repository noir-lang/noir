use std::future::{self, Future};

use async_lsp::{ErrorCode, ResponseError};
use nargo::{
    insert_all_files_for_workspace_into_file_manager,
    ops::{run_test, TestStatus},
    prepare_package,
};
use nargo_toml::{find_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{
    check_crate, file_manager_with_stdlib, CompileOptions, NOIR_ARTIFACT_VERSION_STRING,
};
use noirc_frontend::hir::FunctionNameMatch;

use crate::{
    parse_diff,
    types::{NargoTestRunParams, NargoTestRunResult},
    LspState,
};

pub(crate) fn on_test_run_request(
    state: &mut LspState,
    params: NargoTestRunParams,
) -> impl Future<Output = Result<NargoTestRunResult, ResponseError>> {
    future::ready(on_test_run_request_inner(state, params))
}

fn on_test_run_request_inner(
    state: &mut LspState,
    params: NargoTestRunParams,
) -> Result<NargoTestRunResult, ResponseError> {
    let root_path = state.root_path.as_deref().ok_or_else(|| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "Could not find project root")
    })?;

    let toml_path = find_package_manifest(root_path, root_path).map_err(|err| {
        // If we cannot find a manifest, we can't run the test
        ResponseError::new(ErrorCode::REQUEST_FAILED, err)
    })?;

    let crate_name = params.id.crate_name();
    let function_name = params.id.function_name();

    let workspace = resolve_workspace_from_toml(
        &toml_path,
        PackageSelection::Selected(crate_name.clone()),
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )
    .map_err(|err| {
        // If we found a manifest, but the workspace is invalid, we raise an error about it
        ResponseError::new(ErrorCode::REQUEST_FAILED, err)
    })?;

    let mut workspace_file_manager = file_manager_with_stdlib(&workspace.root_dir);
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_diff(&workspace_file_manager, state);

    // Since we filtered on crate name, this should be the only item in the iterator
    match workspace.into_iter().next() {
        Some(package) => {
            let (mut context, crate_id) =
                prepare_package(&workspace_file_manager, &parsed_files, package);
            if check_crate(&mut context, crate_id, false, false, false).is_err() {
                let result = NargoTestRunResult {
                    id: params.id.clone(),
                    result: "error".to_string(),
                    message: Some("The project failed to compile".into()),
                };
                return Ok(result);
            };

            let test_functions = context.get_all_test_functions_in_crate_matching(
                &crate_id,
                FunctionNameMatch::Exact(function_name),
            );

            let (_, test_function) = test_functions.into_iter().next().ok_or_else(|| {
                ResponseError::new(
                    ErrorCode::REQUEST_FAILED,
                    format!("Could not locate test named: {function_name} in {crate_name}"),
                )
            })?;

            let test_result = run_test(
                &state.solver,
                &mut context,
                &test_function,
                false,
                None,
                &CompileOptions::default(),
            );
            let result = match test_result {
                TestStatus::Pass => NargoTestRunResult {
                    id: params.id.clone(),
                    result: "pass".to_string(),
                    message: None,
                },
                TestStatus::Fail { message, .. } => NargoTestRunResult {
                    id: params.id.clone(),
                    result: "fail".to_string(),
                    message: Some(message),
                },
                TestStatus::CompileError(diag) => NargoTestRunResult {
                    id: params.id.clone(),
                    result: "error".to_string(),
                    message: Some(diag.diagnostic.message),
                },
            };
            Ok(result)
        }
        None => Err(ResponseError::new(
            ErrorCode::REQUEST_FAILED,
            format!("Could not locate package named: {crate_name}"),
        )),
    }
}
