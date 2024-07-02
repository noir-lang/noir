use std::future::{self, Future};

use crate::{parse_diff, resolve_workspace_for_source_path};
use crate::{types::GotoDefinitionResult, LspState};
use async_lsp::{ErrorCode, ResponseError};

use fm::PathString;
use lsp_types::request::GotoTypeDefinitionParams;
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse};
use nargo::insert_all_files_for_workspace_into_file_manager;
use noirc_driver::file_manager_with_stdlib;

use super::{position_to_location, to_lsp_location};

pub(crate) fn on_goto_definition_request(
    state: &mut LspState,
    params: GotoDefinitionParams,
) -> impl Future<Output = Result<GotoDefinitionResult, ResponseError>> {
    let result = on_goto_definition_inner(state, params, false);
    future::ready(result)
}

pub(crate) fn on_goto_type_definition_request(
    state: &mut LspState,
    params: GotoTypeDefinitionParams,
) -> impl Future<Output = Result<GotoDefinitionResult, ResponseError>> {
    let result = on_goto_definition_inner(state, params, true);
    future::ready(result)
}

fn on_goto_definition_inner(
    state: &mut LspState,
    params: GotoDefinitionParams,
    return_type_location_instead: bool,
) -> Result<GotoDefinitionResult, ResponseError> {
    let file_path =
        params.text_document_position_params.text_document.uri.to_file_path().map_err(|_| {
            ResponseError::new(ErrorCode::REQUEST_FAILED, "URI is not a valid file path")
        })?;

    let workspace = resolve_workspace_for_source_path(file_path.as_path()).unwrap();
    let package = workspace.members.first().unwrap();

    let mut workspace_file_manager = file_manager_with_stdlib(&workspace.root_dir);
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_diff(&workspace_file_manager, state);

    let (mut context, crate_id) =
        nargo::prepare_package(&workspace_file_manager, &parsed_files, package);

    let package_root_path = package.root_dir.as_os_str().to_string_lossy().into_owned();
    let interner = if let Some(def_interner) = state.cached_definitions.get(&package_root_path) {
        def_interner
    } else {
        // We ignore the warnings and errors produced by compilation while resolving the definition
        let _ = noirc_driver::check_crate(&mut context, crate_id, false, false, false);
        &context.def_interner
    };

    let files = workspace_file_manager.as_file_map();
    let file_path = PathString::from(file_path);
    let search_for_location =
        position_to_location(files, &file_path, &params.text_document_position_params.position)?;

    let goto_definition_response = interner
        .get_definition_location_from(search_for_location, return_type_location_instead)
        .and_then(|found_location| {
            let file_id = found_location.file;
            let definition_position = to_lsp_location(files, file_id, found_location.span)?;
            let response = GotoDefinitionResponse::from(definition_position).to_owned();
            Some(response)
        });

    Ok(goto_definition_response)
}

#[cfg(test)]
mod goto_definition_tests {
    use std::panic;

    use crate::test_utils;
    use lsp_types::{Position, Range};
    use tokio::test;

    use super::*;

    #[test]
    async fn test_on_goto_definition() {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("go_to_definition").await;

        let params = GotoDefinitionParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: noir_text_document },
                position: Position { line: 9, character: 12 }, // Right at the beginning of "another_function"
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let response: GotoDefinitionResponse = on_goto_definition_request(&mut state, params)
            .await
            .expect("Could execute on_goto_definition_request")
            .expect("Didn't get a goto definition response");

        if let GotoDefinitionResponse::Scalar(location) = response {
            assert_eq!(
                location.range,
                Range {
                    start: Position { line: 4, character: 3 },
                    end: Position { line: 4, character: 19 },
                }
            );
        } else {
            panic!("Expected a scalar response");
        };
    }
}
