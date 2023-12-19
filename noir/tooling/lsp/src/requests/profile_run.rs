use std::{
    collections::{BTreeMap, HashMap},
    future::{self, Future},
};

use acvm::ExpressionWidth;
use async_lsp::{ErrorCode, ResponseError};
use nargo::artifacts::debug::DebugArtifact;
use nargo_toml::{find_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{CompileOptions, DebugFile, NOIR_ARTIFACT_VERSION_STRING};
use noirc_errors::{debug_info::OpCodesCount, Location};

use crate::{
    types::{NargoProfileRunParams, NargoProfileRunResult},
    LspState,
};
use fm::FileId;

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
        Some(_package) => {
            let (binary_packages, contract_packages): (Vec<_>, Vec<_>) = workspace
                .into_iter()
                .filter(|package| !package.is_library())
                .cloned()
                .partition(|package| package.is_binary());

            let expression_width = ExpressionWidth::Bounded { width: 3 };

            let (compiled_programs, compiled_contracts) = nargo::ops::compile_workspace(
                &workspace,
                &binary_packages,
                &contract_packages,
                expression_width,
                &CompileOptions::default(),
            )
            .map_err(|err| ResponseError::new(ErrorCode::REQUEST_FAILED, err))?;

            let mut opcodes_counts: HashMap<Location, OpCodesCount> = HashMap::new();
            let mut file_map: BTreeMap<FileId, DebugFile> = BTreeMap::new();
            for compiled_program in &compiled_programs {
                let span_opcodes = compiled_program.debug.count_span_opcodes();
                let debug_artifact: DebugArtifact = compiled_program.clone().into();
                opcodes_counts.extend(span_opcodes);
                file_map.extend(debug_artifact.file_map);
            }

            for compiled_contract in &compiled_contracts {
                let functions = &compiled_contract.functions;
                let debug_artifact: DebugArtifact = compiled_contract.clone().into();
                file_map.extend(debug_artifact.file_map);
                for contract_function in functions {
                    let span_opcodes = contract_function.debug.count_span_opcodes();
                    opcodes_counts.extend(span_opcodes);
                }
            }

            let result = NargoProfileRunResult { file_map, opcodes_counts };

            Ok(result)
        }
        None => Err(ResponseError::new(
            ErrorCode::REQUEST_FAILED,
            format!("Could not locate package named: {crate_name}"),
        )),
    }
}
