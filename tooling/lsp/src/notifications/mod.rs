use std::ops::ControlFlow;

use async_lsp::{ErrorCode, LanguageClient, ResponseError};
use nargo::prepare_package;
use noirc_driver::check_crate;
use noirc_errors::{DiagnosticKind, FileDiagnostic};

use crate::requests::collect_lenses_for_package;
use crate::types::{
    notification, Diagnostic, DiagnosticSeverity, DidChangeConfigurationParams,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, InitializedParams, NargoPackageTests, PublishDiagnosticsParams,
};

use crate::{
    byte_span_to_range, get_package_tests_in_crate, resolve_workspace_for_source_path, LspState,
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
    ControlFlow::Continue(())
}

pub(super) fn on_did_change_text_document(
    state: &mut LspState,
    params: DidChangeTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    let text = params.content_changes.into_iter().next().unwrap().text;
    state.input_files.insert(params.text_document.uri.to_string(), text.clone());

    let (mut context, crate_id) = nargo::prepare_source(text);
    let _ = check_crate(&mut context, crate_id, false, false);

    let workspace = match resolve_workspace_for_source_path(
        params.text_document.uri.to_file_path().unwrap().as_path(),
    ) {
        Ok(workspace) => workspace,
        Err(lsp_error) => {
            return ControlFlow::Break(Err(ResponseError::new(
                ErrorCode::REQUEST_FAILED,
                format!("{}", lsp_error),
            )
            .into()))
        }
    };
    let package = match workspace.members.first() {
        Some(package) => package,
        None => {
            return ControlFlow::Break(Err(ResponseError::new(
                ErrorCode::REQUEST_FAILED,
                "Selected workspace has no members",
            )
            .into()))
        }
    };

    let lenses = collect_lenses_for_package(&context, crate_id, &workspace, package, None);

    state.cached_lenses.insert(params.text_document.uri.to_string(), lenses);

    ControlFlow::Continue(())
}

pub(super) fn on_did_close_text_document(
    state: &mut LspState,
    params: DidCloseTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    state.input_files.remove(&params.text_document.uri.to_string());
    state.cached_lenses.remove(&params.text_document.uri.to_string());
    ControlFlow::Continue(())
}

pub(super) fn on_did_save_text_document(
    state: &mut LspState,
    params: DidSaveTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    let file_path = match params.text_document.uri.to_file_path() {
        Ok(file_path) => file_path,
        Err(()) => {
            return ControlFlow::Break(Err(ResponseError::new(
                ErrorCode::REQUEST_FAILED,
                "URI is not a valid file path",
            )
            .into()))
        }
    };

    let workspace = match resolve_workspace_for_source_path(&file_path) {
        Ok(value) => value,
        Err(lsp_error) => {
            return ControlFlow::Break(Err(ResponseError::new(
                ErrorCode::REQUEST_FAILED,
                format!("{}", lsp_error),
            )
            .into()))
        }
    };

    let diagnostics: Vec<_> = workspace
        .into_iter()
        .flat_map(|package| -> Vec<Diagnostic> {
            let (mut context, crate_id) = prepare_package(package);

            let file_diagnostics = match check_crate(&mut context, crate_id, false, false) {
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
            state.cached_lenses.insert(params.text_document.uri.to_string(), collected_lenses);

            let fm = &context.file_manager;
            let files = fm.as_file_map();

            file_diagnostics
                .into_iter()
                .filter_map(|FileDiagnostic { file_id, diagnostic, call_stack: _ }| {
                    // Ignore diagnostics for any file that wasn't the file we saved
                    // TODO: In the future, we could create "related" diagnostics for these files
                    if fm.path(file_id) != file_path {
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
                    };
                    Some(Diagnostic {
                        range,
                        severity: Some(severity),
                        message: diagnostic.message,
                        ..Default::default()
                    })
                })
                .collect()
        })
        .collect();

    let _ = state.client.publish_diagnostics(PublishDiagnosticsParams {
        uri: params.text_document.uri,
        version: None,
        diagnostics,
    });

    ControlFlow::Continue(())
}

pub(super) fn on_exit(
    _state: &mut LspState,
    _params: (),
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}
