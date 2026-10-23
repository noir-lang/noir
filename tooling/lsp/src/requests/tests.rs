use std::sync::Arc;

use crate::insert_all_files_for_workspace_into_file_manager;
use async_lsp::lsp_types::{LogMessageParams, MessageType};
use async_lsp::{ErrorCode, LanguageClient, ResponseError};
use nargo_toml::{PackageSelection, find_package_manifest, resolve_workspace_from_toml};
use noirc_driver::{CompileOptions, NOIR_ARTIFACT_VERSION_STRING};

use crate::{
    LspState, add_workspace_root_file, get_package_tests_in_crate, parse_diff, prepare_workspace,
    types::{NargoPackageTests, NargoTestsParams, NargoTestsResult},
};

pub(crate) fn on_tests_request(
    state: &mut LspState,
    params: NargoTestsParams,
) -> Result<NargoTestsResult, ResponseError> {
    on_tests_request_inner(state, params)
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

    let mut workspace_file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(
        state,
        &workspace,
        &mut workspace_file_manager,
    );
    add_workspace_root_file(&workspace, &mut workspace_file_manager);
    let parsed_files = Arc::new(parse_diff(&workspace_file_manager, state));
    let workspace_file_manager = Arc::new(workspace_file_manager);

    let prepared = prepare_workspace(&workspace_file_manager, &parsed_files, &workspace);
    // We ignore the warnings and errors produced by compilation for producing tests
    // because we can still get the test functions even if compilation fails
    let options = CompileOptions::default();
    let (context, _) = prepared.context.check_crate(
        prepared.root_crate_id,
        options.frontend_options(),
        &mut options.comptime_io(),
    );

    let package_tests: Vec<_> = workspace
        .into_iter()
        .filter_map(|package| {
            let crate_id = prepared.package_crate_ids.get(&package.root_dir)?;
            // We don't add test headings for a package if it contains no `#[test]` functions
            get_package_tests_in_crate(&context, crate_id, &package.name)
                .map(|tests| NargoPackageTests { package: package.name.to_string(), tests })
        })
        .collect();

    if package_tests.is_empty() { Ok(None) } else { Ok(Some(package_tests)) }
}
