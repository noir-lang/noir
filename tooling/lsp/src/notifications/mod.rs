use std::ops::ControlFlow;

use async_lsp::{ErrorCode, LanguageClient, ResponseError};
use nargo::prepare_package;
use nargo_toml::{find_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::check_crate;
use noirc_errors::{DiagnosticKind, FileDiagnostic};

use crate::types::{
    notification, Diagnostic, DiagnosticSeverity, DidChangeConfigurationParams,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, InitializedParams, LogMessageParams, MessageType, NargoPackageTests,
    PublishDiagnosticsParams,
};

use crate::{byte_span_to_range, get_non_stdlib_asset, get_package_tests_in_crate, LspState};

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
    _state: &mut LspState,
    _params: DidOpenTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

pub(super) fn on_did_change_text_document(
    _state: &mut LspState,
    _params: DidChangeTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

pub(super) fn on_did_close_text_document(
    _state: &mut LspState,
    _params: DidCloseTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
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

    let root_path = match &state.root_path {
        Some(root) => root,
        None => {
            return ControlFlow::Break(Err(ResponseError::new(
                ErrorCode::REQUEST_FAILED,
                "Could not find project root",
            )
            .into()));
        }
    };

    let toml_path = match find_package_manifest(root_path, &file_path) {
        Ok(toml_path) => toml_path,
        Err(err) => {
            // If we cannot find a manifest, we log a warning but return no diagnostics
            // We can reconsider this when we can build a file without the need for a Nargo.toml file to resolve deps
            let _ = state.client.log_message(LogMessageParams {
                typ: MessageType::WARNING,
                message: format!("{err}"),
            });
            return ControlFlow::Continue(());
        }
    };
    let workspace = match resolve_workspace_from_toml(&toml_path, PackageSelection::All) {
        Ok(workspace) => workspace,
        Err(err) => {
            // If we found a manifest, but the workspace is invalid, we raise an error about it
            return ControlFlow::Break(Err(ResponseError::new(
                ErrorCode::REQUEST_FAILED,
                format!("{err}"),
            )
            .into()));
        }
    };

    let diagnostics: Vec<_> = workspace
        .into_iter()
        .flat_map(|package| -> Vec<Diagnostic> {
            let (mut context, crate_id) = prepare_package(package, Box::new(get_non_stdlib_asset));

            let file_diagnostics = match check_crate(&mut context, crate_id, false) {
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

    // We need to refresh lenses when we compile since that's the only time they can be accurately reflected
    std::mem::drop(state.client.code_lens_refresh(()));

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
