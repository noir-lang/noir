use std::future::{self, Future};

use async_lsp::{ErrorCode, LanguageClient, ResponseError};
use lsp_types::{LogMessageParams, MessageType};
use nargo::insert_all_files_for_workspace_into_file_manager;
use nargo_toml::{find_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{check_crate, file_manager_with_stdlib, NOIR_ARTIFACT_VERSION_STRING};

use crate::{
    get_package_tests_in_crate, parse_diff,
    types::{NargoPackageTests, NargoTestsParams, NargoTestsResult},
    LspState,
};

pub(crate) fn on_tests_request(
    state: &mut LspState,
    params: NargoTestsParams,
) -> impl Future<Output = Result<NargoTestsResult, ResponseError>> {
    future::ready(on_tests_request_inner(state, params))
}

fn on_tests_request_inner(
    state: &mut LspState,
    _params: NargoTestsParams,
) -> Result<NargoTestsResult, ResponseError> {
    let root_path = state.root_path.as_deref().ok_or_else(|| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "Could not find project root")
    })?;

    let toml_path = match find_package_manifest(root_path, root_path) {
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

    let workspace = resolve_workspace_from_toml(
        &toml_path,
        PackageSelection::All,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )
    .map_err(|err| {
        // If we found a manifest, but the workspace is invalid, we raise an error about it
        ResponseError::new(ErrorCode::REQUEST_FAILED, err)
    })?;

    let mut workspace_file_manager = file_manager_with_stdlib(&workspace.root_dir);
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_diff(&workspace_file_manager, state);

    let package_tests: Vec<_> = workspace
        .into_iter()
        .filter_map(|package| {
            let (mut context, crate_id) =
                crate::prepare_package(&workspace_file_manager, &parsed_files, package);
            // We ignore the warnings and errors produced by compilation for producing tests
            // because we can still get the test functions even if compilation fails
            let _ = check_crate(&mut context, crate_id, false, false, false, None);

            // We don't add test headings for a package if it contains no `#[test]` functions
            get_package_tests_in_crate(&context, &crate_id, &package.name)
                .map(|tests| NargoPackageTests { package: package.name.to_string(), tests })
        })
        .collect();

    if package_tests.is_empty() {
        Ok(None)
    } else {
        Ok(Some(package_tests))
    }
}
