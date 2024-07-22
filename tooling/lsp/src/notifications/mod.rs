use std::ops::ControlFlow;

use crate::insert_all_files_for_workspace_into_file_manager;
use async_lsp::{ErrorCode, LanguageClient, ResponseError};
use noirc_driver::{check_crate, file_manager_with_stdlib};
use noirc_errors::{DiagnosticKind, FileDiagnostic};

use crate::types::{
    notification, Diagnostic, DiagnosticSeverity, DidChangeConfigurationParams,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, InitializedParams, NargoPackageTests, PublishDiagnosticsParams,
};

use crate::{
    byte_span_to_range, get_package_tests_in_crate, parse_diff, resolve_workspace_for_source_path,
    LspState,
};

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

pub(super) fn on_did_open_text_document(
    state: &mut LspState,
    params: DidOpenTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    state.input_files.insert(params.text_document.uri.to_string(), params.text_document.text);

    let document_uri = params.text_document.uri;
    let only_process_document_uri_package = false;
    let output_diagnostics = true;

    match process_workspace_for_noir_document(
        state,
        document_uri,
        only_process_document_uri_package,
        output_diagnostics,
    ) {
        Ok(_) => {
            state.open_documents_count += 1;
            ControlFlow::Continue(())
        }
        Err(err) => ControlFlow::Break(Err(err)),
    }
}

pub(super) fn on_did_change_text_document(
    state: &mut LspState,
    params: DidChangeTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    let text = params.content_changes.into_iter().next().unwrap().text;
    state.input_files.insert(params.text_document.uri.to_string(), text.clone());

    let document_uri = params.text_document.uri;
    let only_process_document_uri_package = true;
    let output_diagnotics = false;

    match process_workspace_for_noir_document(
        state,
        document_uri,
        only_process_document_uri_package,
        output_diagnotics,
    ) {
        Ok(_) => ControlFlow::Continue(()),
        Err(err) => ControlFlow::Break(Err(err)),
    }
}

pub(super) fn on_did_close_text_document(
    state: &mut LspState,
    params: DidCloseTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    state.input_files.remove(&params.text_document.uri.to_string());
    state.cached_lenses.remove(&params.text_document.uri.to_string());

    state.open_documents_count -= 1;

    if state.open_documents_count == 0 {
        state.cached_definitions.clear();
    }

    let document_uri = params.text_document.uri;
    let only_process_document_uri_package = true;
    let output_diagnotics = false;

    match process_workspace_for_noir_document(
        state,
        document_uri,
        only_process_document_uri_package,
        output_diagnotics,
    ) {
        Ok(_) => ControlFlow::Continue(()),
        Err(err) => ControlFlow::Break(Err(err)),
    }
}

pub(super) fn on_did_save_text_document(
    state: &mut LspState,
    params: DidSaveTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    let document_uri = params.text_document.uri;
    let only_process_document_uri_package = false;
    let output_diagnotics = true;

    match process_workspace_for_noir_document(
        state,
        document_uri,
        only_process_document_uri_package,
        output_diagnotics,
    ) {
        Ok(_) => ControlFlow::Continue(()),
        Err(err) => ControlFlow::Break(Err(err)),
    }
}

// Given a Noir document, find the workspace it's contained in (an assumed workspace is created if
// it's only contained in a package), then type-checks the workspace's packages,
// caching code lenses and type definitions, and notifying about compilation errors.
pub(crate) fn process_workspace_for_noir_document(
    state: &mut LspState,
    document_uri: lsp_types::Url,
    only_process_document_uri_package: bool,
    output_diagnostics: bool,
) -> Result<(), async_lsp::Error> {
    let file_path = document_uri.to_file_path().map_err(|_| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "URI is not a valid file path")
    })?;

    let workspace =
        resolve_workspace_for_source_path(&file_path, &state.root_path).map_err(|lsp_error| {
            ResponseError::new(ErrorCode::REQUEST_FAILED, lsp_error.to_string())
        })?;

    let mut workspace_file_manager = file_manager_with_stdlib(&workspace.root_dir);
    insert_all_files_for_workspace_into_file_manager(
        state,
        &workspace,
        &mut workspace_file_manager,
    );

    let parsed_files = parse_diff(&workspace_file_manager, state);

    let diagnostics: Vec<_> = workspace
        .into_iter()
        .flat_map(|package| -> Vec<Diagnostic> {
            let package_root_dir: String = package.root_dir.as_os_str().to_string_lossy().into();

            if only_process_document_uri_package && !file_path.starts_with(&package.root_dir) {
                return vec![];
            }

            let (mut context, crate_id) =
                crate::prepare_package(&workspace_file_manager, &parsed_files, package);

            let file_diagnostics = match check_crate(&mut context, crate_id, false, false, None) {
                Ok(((), warnings)) => warnings,
                Err(errors_and_warnings) => errors_and_warnings,
            };

            // We don't add test headings for a package if it contains no `#[test]` functions
            if let Some(tests) = get_package_tests_in_crate(&context, &crate_id, &package.name) {
                let _ = state.client.notify::<notification::NargoUpdateTests>(NargoPackageTests {
                    package: package.name.to_string(),
                    tests,
                });
            }

            let collected_lenses = crate::requests::collect_lenses_for_package(
                &context,
                crate_id,
                &workspace,
                package,
                Some(&file_path),
            );
            state.cached_lenses.insert(document_uri.to_string(), collected_lenses);

            state.cached_definitions.insert(package_root_dir, context.def_interner);

            let fm = &context.file_manager;
            let files = fm.as_file_map();

            if output_diagnostics {
                file_diagnostics
                    .into_iter()
                    .filter_map(|FileDiagnostic { file_id, diagnostic, call_stack: _ }| {
                        // Ignore diagnostics for any file that wasn't the file we saved
                        // TODO: In the future, we could create "related" diagnostics for these files
                        if fm.path(file_id).expect("file must exist to have emitted diagnostic")
                            != file_path
                        {
                            return None;
                        }

                        // TODO: Should this be the first item in secondaries? Should we bail when we find a range?
                        let range = diagnostic
                            .secondaries
                            .into_iter()
                            .filter_map(|sec| byte_span_to_range(files, file_id, sec.span.into()))
                            .last()
                            .unwrap_or_default();

                        let severity = match diagnostic.kind {
                            DiagnosticKind::Error => DiagnosticSeverity::ERROR,
                            DiagnosticKind::Warning => DiagnosticSeverity::WARNING,
                            DiagnosticKind::Info => DiagnosticSeverity::INFORMATION,
                            DiagnosticKind::Bug => DiagnosticSeverity::WARNING,
                        };
                        Some(Diagnostic {
                            range,
                            severity: Some(severity),
                            message: diagnostic.message,
                            ..Default::default()
                        })
                    })
                    .collect()
            } else {
                vec![]
            }
        })
        .collect();

    if output_diagnostics {
        let _ = state.client.publish_diagnostics(PublishDiagnosticsParams {
            uri: document_uri,
            version: None,
            diagnostics,
        });
    }

    Ok(())
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
    use lsp_types::{
        InlayHintLabel, InlayHintParams, Position, TextDocumentContentChangeEvent,
        TextDocumentIdentifier, TextDocumentItem, VersionedTextDocumentIdentifier,
        WorkDoneProgressParams,
    };
    use tokio::test;

    #[test]
    async fn test_caches_open_files() {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("inlay_hints").await;

        // Open the document, fake the text to be empty
        on_did_open_text_document(
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
        on_did_change_text_document(
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
                range: lsp_types::Range {
                    start: lsp_types::Position { line: 0, character: 0 },
                    end: lsp_types::Position { line: 1, character: 0 },
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
