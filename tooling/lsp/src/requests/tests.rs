use std::future::{self, Future};

use async_lsp::{ErrorCode, LanguageClient, ResponseError};
use lsp_types::{LogMessageParams, MessageType};
use nargo::prepare_package;
use nargo_toml::{find_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::check_crate;

use crate::{
    get_non_stdlib_asset, get_package_tests_in_crate,
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

    let workspace =
        resolve_workspace_from_toml(&toml_path, PackageSelection::All).map_err(|err| {
            // If we found a manifest, but the workspace is invalid, we raise an error about it
            ResponseError::new(ErrorCode::REQUEST_FAILED, err)
        })?;

    let mut package_tests = Vec::new();

    for package in &workspace {
        let (mut context, crate_id) = prepare_package(package, Box::new(get_non_stdlib_asset));
        // We ignore the warnings and errors produced by compilation for producing tests
        // because we can still get the test functions even if compilation fails
        let _ = check_crate(&mut context, crate_id, false);

        // We don't add test headings for a package if it contains no `#[test]` functions
        if let Some(tests) = get_package_tests_in_crate(&context, &crate_id, &package.name) {
            package_tests.push(NargoPackageTests { package: package.name.to_string(), tests });
        }
    }

    if package_tests.is_empty() {
        Ok(None)
    } else {
        Ok(Some(package_tests))
    }
}
