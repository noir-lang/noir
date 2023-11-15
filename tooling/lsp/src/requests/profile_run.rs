use std::future::{self, Future};

use acvm::{acir::circuit::Opcode, Language};
use async_lsp::{ErrorCode, ResponseError};
use nargo_toml::{find_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{CompileOptions, NOIR_ARTIFACT_VERSION_STRING};

use crate::{
    types::{NargoProfileRunParams, NargoProfileRunResult},
    LspState,
};

pub(crate) fn on_profile_run_request(
    state: &mut LspState,
    params: NargoProfileRunParams,
) -> impl Future<Output = Result<NargoProfileRunResult, ResponseError>> {
    future::ready(on_profile_run_request_inner(state, params))
}

fn on_profile_run_request_inner(
    state: &LspState,
    params: NargoProfileRunParams,
) -> Result<NargoProfileRunResult, ResponseError> {
    let root_path = state.root_path.as_deref().ok_or_else(|| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "Could not find project root")
    })?;

    let toml_path = find_package_manifest(root_path, root_path).map_err(|err| {
        // If we cannot find a manifest, we can't run the test
        ResponseError::new(ErrorCode::REQUEST_FAILED, err)
    })?;

    let crate_name = params.package;
    // let function_name = params.id.function_name();

    let workspace = resolve_workspace_from_toml(
        &toml_path,
        PackageSelection::DefaultOrAll,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )
    .map_err(|err| {
        // If we found a manifest, but the workspace is invalid, we raise an error about it
        ResponseError::new(ErrorCode::REQUEST_FAILED, err)
    })?;

    // Since we filtered on crate name, this should be the only item in the iterator
    match workspace.into_iter().next() {
        Some(package) => {
            // let (mut _context, crate_id) = prepare_package(package, Box::new(get_non_stdlib_asset));

            // let (_binary_packages, _contract_packages): (Vec<_>, Vec<_>) = workspace
            //     .into_iter()
            //     .filter(|package| !package.is_library())
            //     .cloned()
            //     .partition(|package| package.is_binary());

            let is_opcode_supported = |_opcode: &Opcode| true;
            let np_language = Language::PLONKCSat { width: 3 };
            let compilation_result = nargo::ops::compile_program(
                &workspace,
                package,
                &CompileOptions::default(),
                np_language,
                &is_opcode_supported,
            );

            let compiled_program = compilation_result.1.unwrap().0;

            let opcodes_counts = compiled_program.debug.count_span_opcodes();

            let result = NargoProfileRunResult { compiled_program, opcodes_counts };
            Ok(result)
        }
        None => Err(ResponseError::new(
            ErrorCode::REQUEST_FAILED,
            format!("Could not locate package named: {crate_name}"),
        )),
    }
}
