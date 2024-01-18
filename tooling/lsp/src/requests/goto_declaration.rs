use std::future::{self, Future};

use crate::types::GotoDeclarationResult;
use crate::LspState;
use crate::{parse_diff, resolve_workspace_for_source_path};
use async_lsp::{ErrorCode, ResponseError};

use lsp_types::request::{GotoDeclarationParams, GotoDeclarationResponse};

use nargo::insert_all_files_for_workspace_into_file_manager;
use noirc_driver::file_manager_with_stdlib;

use super::{position_to_byte_index, to_lsp_location};

pub(crate) fn on_goto_declaration_request(
    state: &mut LspState,
    params: GotoDeclarationParams,
) -> impl Future<Output = Result<GotoDeclarationResult, ResponseError>> {
    let result = on_goto_definition_inner(state, params);
    future::ready(result)
}

fn on_goto_definition_inner(
    state: &mut LspState,
    params: GotoDeclarationParams,
) -> Result<GotoDeclarationResult, ResponseError> {
    let file_path =
        params.text_document_position_params.text_document.uri.to_file_path().map_err(|_| {
            ResponseError::new(ErrorCode::REQUEST_FAILED, "URI is not a valid file path")
        })?;

    let workspace = resolve_workspace_for_source_path(file_path.as_path()).unwrap();
    let package = workspace.members.first().unwrap();

    let package_root_path: String = package.root_dir.as_os_str().to_string_lossy().into();

    let mut workspace_file_manager = file_manager_with_stdlib(&workspace.root_dir);
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_diff(&workspace_file_manager, state);

    let (mut context, crate_id) =
        nargo::prepare_package(&workspace_file_manager, &parsed_files, package);

    let interner;
    if let Some(def_interner) = state.cached_definitions.get(&package_root_path) {
        interner = def_interner;
    } else {
        // We ignore the warnings and errors produced by compilation while resolving the definition
        let _ = noirc_driver::check_crate(&mut context, crate_id, false, false);
        interner = &context.def_interner;
    }

    let files = context.file_manager.as_file_map();
    let file_id = context.file_manager.name_to_id(file_path.clone()).ok_or(ResponseError::new(
        ErrorCode::REQUEST_FAILED,
        format!("Could not find file in file manager. File path: {:?}", file_path),
    ))?;
    let byte_index =
        position_to_byte_index(files, file_id, &params.text_document_position_params.position)
            .map_err(|err| {
                ResponseError::new(
                    ErrorCode::REQUEST_FAILED,
                    format!("Could not convert position to byte index. Error: {:?}", err),
                )
            })?;

    let search_for_location = noirc_errors::Location {
        file: file_id,
        span: noirc_errors::Span::single_char(byte_index as u32),
    };

    let goto_declaration_response =
        interner.get_declaration_location_from(search_for_location).and_then(|found_location| {
            let file_id = found_location.file;
            let definition_position = to_lsp_location(files, file_id, found_location.span)?;
            let response: GotoDeclarationResponse =
                GotoDeclarationResponse::from(definition_position).to_owned();
            Some(response)
        });

    Ok(goto_declaration_response)
}
