use std::{
    collections::HashMap,
    future::{self, Future},
};

use async_lsp::{ErrorCode, ResponseError};
use lsp_types::{
    PrepareRenameResponse, RenameParams, TextDocumentPositionParams, TextEdit, Url, WorkspaceEdit,
};
use nargo::insert_all_files_for_workspace_into_file_manager;
use noirc_driver::file_manager_with_stdlib;

use crate::{parse_diff, resolve_workspace_for_source_path, LspState};

use super::{position_to_byte_index, to_lsp_location};

pub(crate) fn on_prepare_rename_request(
    state: &mut LspState,
    params: TextDocumentPositionParams,
) -> impl Future<Output = Result<Option<PrepareRenameResponse>, ResponseError>> {
    let result = on_prepare_rename_inner(state, params);
    future::ready(result)
}

fn on_prepare_rename_inner(
    state: &mut LspState,
    params: TextDocumentPositionParams,
) -> Result<Option<PrepareRenameResponse>, ResponseError> {
    let file_path = params.text_document.uri.to_file_path().map_err(|_| {
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
        let _ = noirc_driver::check_crate(&mut context, crate_id, false, false, false);
        interner = &context.def_interner;
    }

    let files = context.file_manager.as_file_map();
    let file_id = context.file_manager.name_to_id(file_path.clone()).ok_or(ResponseError::new(
        ErrorCode::REQUEST_FAILED,
        format!("Could not find file in file manager. File path: {:?}", file_path),
    ))?;
    let byte_index = position_to_byte_index(files, file_id, &params.position).map_err(|err| {
        ResponseError::new(
            ErrorCode::REQUEST_FAILED,
            format!("Could not convert position to byte index. Error: {:?}", err),
        )
    })?;

    let search_for_location = noirc_errors::Location {
        file: file_id,
        span: noirc_errors::Span::single_char(byte_index as u32),
    };

    let rename_possible = interner.check_rename_possible(search_for_location);

    let response = PrepareRenameResponse::DefaultBehavior { default_behavior: rename_possible };

    Ok(Some(response))
}

pub(crate) fn on_rename_request(
    state: &mut LspState,
    params: RenameParams,
) -> impl Future<Output = Result<Option<WorkspaceEdit>, ResponseError>> {
    let result = on_rename_inner(state, params);
    future::ready(result)
}

fn on_rename_inner(
    state: &mut LspState,
    params: RenameParams,
) -> Result<Option<WorkspaceEdit>, ResponseError> {
    let file_path =
        params.text_document_position.text_document.uri.to_file_path().map_err(|_| {
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
        let _ = noirc_driver::check_crate(&mut context, crate_id, false, false, true);
        interner = &context.def_interner;
    }

    let files = context.file_manager.as_file_map();
    let file_id = context.file_manager.name_to_id(file_path.clone()).ok_or(ResponseError::new(
        ErrorCode::REQUEST_FAILED,
        format!("Could not find file in file manager. File path: {:?}", file_path),
    ))?;
    let byte_index =
        position_to_byte_index(files, file_id, &params.text_document_position.position).map_err(
            |err| {
                ResponseError::new(
                    ErrorCode::REQUEST_FAILED,
                    format!("Could not convert position to byte index. Error: {:?}", err),
                )
            },
        )?;

    let search_for_location = noirc_errors::Location {
        file: file_id,
        span: noirc_errors::Span::single_char(byte_index as u32),
    };

    let rename_changes = interner.find_rename_symbols_at(search_for_location).map(|locations| {
        let rs = locations.iter().fold(
            HashMap::new(),
            |mut acc: HashMap<Url, Vec<TextEdit>>, location| {
                let file_id = location.file;
                let span = location.span;

                let Some(lsp_location) = to_lsp_location(files, file_id, span) else {
                    return acc;
                };

                let edit =
                    TextEdit { range: lsp_location.range, new_text: params.new_name.clone() };

                acc.entry(lsp_location.uri).or_default().push(edit);

                acc
            },
        );
        rs
    });

    let response =
        WorkspaceEdit { changes: rename_changes, document_changes: None, change_annotations: None };

    Ok(Some(response))
}

#[cfg(test)]
mod rename_tests {
    use super::*;
    use acvm::blackbox_solver::StubbedBlackBoxSolver;
    use async_lsp::ClientSocket;
    use lsp_types::{Position, Range, WorkDoneProgressParams};
    use tokio::test;

    async fn setup() -> (LspState, Url) {
        let client = ClientSocket::new_closed();
        let mut state = LspState::new(&client, StubbedBlackBoxSolver);

        let root_path = std::env::current_dir()
            .unwrap()
            .join("../../test_programs/execution_success/7_function")
            .canonicalize()
            .expect("Could not resolve root path");
        let noir_text_document = Url::from_file_path(root_path.join("src/main.nr").as_path())
            .expect("Could not convert text document path to URI");
        let root_uri = Some(
            Url::from_file_path(root_path.as_path()).expect("Could not convert root path to URI"),
        );

        #[allow(deprecated)]
        let initialize_params = lsp_types::InitializeParams {
            process_id: Default::default(),
            root_path: None,
            root_uri,
            initialization_options: None,
            capabilities: Default::default(),
            trace: Some(lsp_types::TraceValue::Verbose),
            workspace_folders: None,
            client_info: None,
            locale: None,
        };

        let _initialize_response = crate::requests::on_initialize(&mut state, initialize_params)
            .await
            .expect("Could not initialize LSP server");

        (state, noir_text_document)
    }

    #[test]
    async fn test_on_prepare_rename_request_cannot_be_applied() {
        let (mut state, noir_text_document) = setup().await;

        let params = TextDocumentPositionParams {
            text_document: lsp_types::TextDocumentIdentifier { uri: noir_text_document },
            position: lsp_types::Position { line: 0, character: 0 }, // This is at the "f" of an "fn" keyword
        };

        let response = on_prepare_rename_request(&mut state, params)
            .await
            .expect("Could not execute on_prepare_rename_request");

        assert_eq!(
            response,
            Some(PrepareRenameResponse::DefaultBehavior { default_behavior: false })
        );
    }

    #[test]
    async fn test_on_prepare_rename_request_can_be_applied_at_function_declaration() {
        let (mut state, noir_text_document) = setup().await;

        let params = TextDocumentPositionParams {
            text_document: lsp_types::TextDocumentIdentifier { uri: noir_text_document },
            position: lsp_types::Position { line: 70, character: 3 }, // This is at the "t" of "test_multiple4"
        };

        let response = on_prepare_rename_request(&mut state, params)
            .await
            .expect("Could not execute on_prepare_rename_request");

        assert_eq!(
            response,
            Some(PrepareRenameResponse::DefaultBehavior { default_behavior: true })
        );
    }

    #[test]
    async fn test_on_prepare_rename_request_can_be_applied_at_function_reference() {
        let (mut state, noir_text_document) = setup().await;

        let params = TextDocumentPositionParams {
            text_document: lsp_types::TextDocumentIdentifier { uri: noir_text_document },
            position: lsp_types::Position { line: 79, character: 4 }, // This is at the "t" of "test_multiple4"
        };

        let response = on_prepare_rename_request(&mut state, params)
            .await
            .expect("Could not execute on_prepare_rename_request");

        assert_eq!(
            response,
            Some(PrepareRenameResponse::DefaultBehavior { default_behavior: true })
        );
    }

    #[test]
    async fn test_on_rename_request() {
        let (mut state, noir_text_document) = setup().await;

        let params = RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: noir_text_document },
                position: lsp_types::Position { line: 79, character: 4 }, // This is at the "t" of "test_multiple4"
            },
            new_name: "renamed_test_multiple4".to_string(),
            work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
        };

        let response = on_rename_request(&mut state, params)
            .await
            .expect("Could not execute on_prepare_rename_request")
            .unwrap();

        let changes = response.changes.expect("Expected to find rename changes");
        let mut changes: Vec<Range> = changes.values().flatten().map(|edit| edit.range).collect();
        changes.sort_by_key(|range| range.start.line);
        assert_eq!(
            changes,
            vec![
                Range {
                    start: Position { line: 70, character: 3 },
                    end: Position { line: 70, character: 17 },
                },
                Range {
                    start: Position { line: 79, character: 4 },
                    end: Position { line: 79, character: 18 },
                },
                Range {
                    start: Position { line: 94, character: 4 },
                    end: Position { line: 94, character: 18 },
                },
            ]
        );
    }
}
