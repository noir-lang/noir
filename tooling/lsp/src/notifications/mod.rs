use std::collections::BTreeMap;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::str::FromStr as _;

use crate::events::{
    ProcessWorkspaceEvent, ProcessWorkspaceForSingleFileChangeEvent, on_process_workspace_event,
    on_process_workspace_for_single_file_change,
};
use crate::insert_all_files_for_workspace_into_file_manager;
use async_lsp::lsp_types::Url;
use async_lsp::{ErrorCode, ResponseError};
use nargo::package::{Package, PackageType};
use nargo::workspace::Workspace;
use noirc_driver::{CrateName, NOIR_ARTIFACT_VERSION_STRING};

use crate::types::{
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, InitializedParams,
};

use crate::{LspState, parse_diff, resolve_workspace_for_source_path};

pub(super) fn on_initialized(
    _state: &mut LspState,
    _params: InitializedParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

pub(super) fn on_did_change_configuration(
    _state: &mut LspState,
    _params: DidChangeConfigurationParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

pub(crate) fn on_did_open_text_document(
    state: &mut LspState,
    params: DidOpenTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    state.input_files.insert(params.text_document.uri.to_string(), params.text_document.text);

    let document_uri = params.text_document.uri;

    match handle_text_document_open_or_close_notification(state, document_uri) {
        Ok(_) => ControlFlow::Continue(()),
        Err(err) => ControlFlow::Break(Err(err)),
    }
}

pub(super) fn on_did_change_text_document(
    state: &mut LspState,
    params: DidChangeTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    let text = params.content_changes.into_iter().next().unwrap().text;
    state.input_files.insert(params.text_document.uri.to_string(), text.clone());
    state.workspace_symbol_cache.reprocess_uri(&params.text_document.uri);

    let document_uri = params.text_document.uri;

    match handle_on_did_change_text_document_notification(state, document_uri, &text) {
        Ok(_) => ControlFlow::Continue(()),
        Err(err) => ControlFlow::Break(Err(err)),
    }
}

pub(super) fn on_did_close_text_document(
    state: &mut LspState,
    params: DidCloseTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    state.input_files.remove(&params.text_document.uri.to_string());
    state.workspace_symbol_cache.reprocess_uri(&params.text_document.uri);

    let document_uri = params.text_document.uri;

    match handle_text_document_open_or_close_notification(state, document_uri) {
        Ok(_) => ControlFlow::Continue(()),
        Err(err) => ControlFlow::Break(Err(err)),
    }
}

pub(super) fn on_did_save_text_document(
    state: &mut LspState,
    params: DidSaveTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    let workspace = match workspace_from_document_uri(params.text_document.uri) {
        Ok(workspace) => workspace,
        Err(err) => return ControlFlow::Break(Err(err)),
    };

    match process_workspace(state, workspace) {
        Ok(_) => ControlFlow::Continue(()),
        Err(err) => ControlFlow::Break(Err(err)),
    }
}

fn handle_text_document_open_or_close_notification(
    state: &mut LspState,
    document_uri: Url,
) -> Result<(), async_lsp::Error> {
    let workspace = workspace_from_document_uri(document_uri.clone())?;

    if state.package_cache.contains_key(&workspace.root_dir) {
        Ok(())
    } else {
        // If it's the first time we see this package, show diagnostics.
        // This can happen for example when a user opens a Noir file in a package for the first time.
        process_workspace(state, workspace)
    }
}

fn handle_on_did_change_text_document_notification(
    state: &mut LspState,
    document_uri: Url,
    text: &str,
) -> Result<(), async_lsp::Error> {
    let workspace = workspace_from_document_uri(document_uri.clone())?;

    if state.package_cache.contains_key(&workspace.root_dir) {
        process_workspace_for_single_file_change(state, workspace, document_uri, text)
    } else {
        // If it's the first time we see this package, show diagnostics.
        // This can happen for example when a user opens a Noir file in a package for the first time.
        process_workspace(state, workspace)
    }
}

pub(crate) fn workspace_from_document_uri(
    document_uri: Url,
) -> Result<Workspace, async_lsp::Error> {
    if document_uri.scheme() == "noir-std" {
        Ok(fake_stdlib_workspace())
    } else {
        let file_path = document_uri.to_file_path().map_err(|_| {
            ResponseError::new(ErrorCode::REQUEST_FAILED, "URI is not a valid file path")
        })?;

        let workspace = resolve_workspace_for_source_path(&file_path).map_err(|lsp_error| {
            ResponseError::new(ErrorCode::REQUEST_FAILED, lsp_error.to_string())
        })?;

        Ok(workspace)
    }
}

// Given a Noir document, find the workspace it's contained in (an assumed workspace is created if
// it's only contained in a package), then type-checks the workspace's packages,
// caching type definitions, and notifying about compilation errors if `output_diagnostics` is true.
pub(crate) fn process_workspace(
    state: &mut LspState,
    workspace: Workspace,
) -> Result<(), async_lsp::Error> {
    // Here we don't actually do the processing. Instead, we queue an event for that.
    // The reason is that according to the LSP spec, notifications should be handled quickly
    // as otherwise they block the main UI thread.

    start_type_checking(state);

    let mut file_manager = workspace.new_file_manager();
    if workspace.is_assumed {
        let package = workspace.members.first().unwrap();
        file_manager.add_file_with_source_canonical_path(&package.entry_path, String::new());
    } else {
        insert_all_files_for_workspace_into_file_manager(state, &workspace, &mut file_manager);
    }

    let parsed_files = parse_diff(&file_manager, state);

    let client = state.client.clone();

    if state.test_mode {
        let event = ProcessWorkspaceEvent { workspace, file_manager, parsed_files };
        on_process_workspace_event(state, event);
    } else {
        tokio::spawn(async move {
            let event = ProcessWorkspaceEvent { workspace, file_manager, parsed_files };
            let _ = client.emit(event);
        });
    }

    Ok(())
}

/// Type-checks a single file that changed by using existing cached data for the workspace/package,
/// such as the cached NodeInterner, CrateGraph and DefMaps.
///
/// This greatly improves the responsiveness of the LSP server when editing files. However,
/// the cost is a slight decrease in autocompletion accuracy. For example, if a struct is removed
/// from the code, it will still existing in the cached data and it will still be offered as
/// autocompletion. Or for example if the compiler refuses to re-process a trait because
/// it's already defined, new methods defined on that trait won't be available for autocompletion.
/// However, this is solved when the file is saved as the entire package is re-processed then.
/// We think this is an acceptable trade-off to improve responsiveness, and it could eventually
/// be further improved.
pub(crate) fn process_workspace_for_single_file_change(
    state: &mut LspState,
    workspace: Workspace,
    file_uri: Url,
    file_source: &str,
) -> Result<(), async_lsp::Error> {
    // Here we don't actually do the processing. Instead, we queue an event for that.
    // The reason is that according to the LSP spec, notifications should be handled quickly
    // as otherwise they block the main UI thread.

    start_type_checking(state);

    let file_source = file_source.to_string();
    let client = state.client.clone();

    if state.test_mode {
        let event = ProcessWorkspaceForSingleFileChangeEvent { workspace, file_uri, file_source };
        on_process_workspace_for_single_file_change(state, event);
    } else {
        tokio::spawn(async move {
            let event =
                ProcessWorkspaceForSingleFileChangeEvent { workspace, file_uri, file_source };
            let _ = client.emit(event);
        });
    }

    Ok(())
}

fn start_type_checking(state: &mut LspState) {
    state.type_check_version = state.type_check_version.wrapping_add(1);

    if !state.test_mode {
        state.pending_type_check_events += 1;
    }
}

pub(crate) fn fake_stdlib_workspace() -> Workspace {
    let assumed_package = Package {
        version: None,
        compiler_required_version: Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
        compiler_required_unstable_features: Vec::new(),
        root_dir: PathBuf::from_str("std").unwrap(),
        package_type: PackageType::Binary,
        entry_path: PathBuf::from_str("fake_entry_path.nr").unwrap(),
        name: CrateName::from_str("fake_std").unwrap(),
        dependencies: BTreeMap::new(),
    };
    Workspace {
        root_dir: PathBuf::from_str("std").unwrap(),
        members: vec![assumed_package],
        selected_package_index: Some(0),
        is_assumed: true,
        target_dir: None,
    }
}

pub(super) fn on_exit(
    _state: &mut LspState,
    _params: (),
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

#[cfg(test)]
mod notification_tests {
    use crate::test_utils;

    use super::*;
    use async_lsp::lsp_types::{
        InlayHintLabel, InlayHintParams, Position, Range, TextDocumentContentChangeEvent,
        TextDocumentIdentifier, TextDocumentItem, VersionedTextDocumentIdentifier,
        WorkDoneProgressParams,
    };
    use tokio::test;

    #[test]
    async fn test_caches_open_files() {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("inlay_hints").await;

        // Open the document, fake the text to be empty
        let _ = on_did_open_text_document(
            &mut state,
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: noir_text_document.clone(),
                    language_id: "noir".to_string(),
                    version: 0,
                    text: "".to_string(),
                },
            },
        );

        // Fake the text to change to "global a = 1;"
        let _ = on_did_change_text_document(
            &mut state,
            DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri: noir_text_document.clone(),
                    version: 1,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    // Should get an inlay hint for ": bool" after "a"
                    text: "global a = true;".to_string(),
                }],
            },
        );

        // Get inlay hints. These should now be relative to the changed text,
        // not the saved file's text.
        let inlay_hints = crate::requests::on_inlay_hint_request(
            &mut state,
            InlayHintParams {
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
                text_document: TextDocumentIdentifier { uri: noir_text_document },
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 1, character: 0 },
                },
            },
        )
        .await
        .expect("Could not execute on_inlay_hint_request")
        .unwrap();

        assert_eq!(inlay_hints.len(), 1);

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, Position { line: 0, character: 8 });

        if let InlayHintLabel::LabelParts(labels) = &inlay_hint.label {
            assert_eq!(labels.len(), 2);
            assert_eq!(labels[0].value, ": ");
            assert_eq!(labels[0].location, None);
            assert_eq!(labels[1].value, "bool");
        } else {
            panic!("Expected InlayHintLabel::LabelParts, got {:?}", inlay_hint.label);
        }
    }
}
