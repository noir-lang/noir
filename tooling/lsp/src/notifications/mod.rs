use std::ops::ControlFlow;

use async_lsp::{ErrorCode, LanguageClient, ResponseError};
use nargo::{insert_all_files_for_workspace_into_file_manager, prepare_package};
use noirc_driver::{check_crate, file_manager_with_stdlib};
use noirc_errors::{DiagnosticKind, FileDiagnostic};

use crate::requests::collect_lenses_for_package;
use crate::types::{
    notification, Diagnostic, DiagnosticSeverity, DidChangeConfigurationParams,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, InitializedParams, NargoPackageTests, PublishDiagnosticsParams,
};

use crate::{
    byte_span_to_range, get_package_tests_in_crate, parse_diff, prepare_source,
    resolve_workspace_for_source_path, LspState,
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

    match process_noir_document(document_uri, state) {
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

    let (mut context, crate_id) = prepare_source(text, state);
    let _ = check_crate(&mut context, crate_id, false, false);

    let workspace = match resolve_workspace_for_source_path(
        params.text_document.uri.to_file_path().unwrap().as_path(),
    ) {
        Ok(workspace) => workspace,
        Err(lsp_error) => {
            return ControlFlow::Break(Err(ResponseError::new(
                ErrorCode::REQUEST_FAILED,
                lsp_error.to_string(),
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

    state.open_documents_count -= 1;

    if state.open_documents_count == 0 {
        state.cached_definitions.clear();
    }

    ControlFlow::Continue(())
}

pub(super) fn on_did_save_text_document(
    state: &mut LspState,
    params: DidSaveTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    let document_uri = params.text_document.uri;

    match process_noir_document(document_uri, state) {
        Ok(_) => ControlFlow::Continue(()),
        Err(err) => ControlFlow::Break(Err(err)),
    }
}

fn process_noir_document(
    document_uri: lsp_types::Url,
    state: &mut LspState,
) -> Result<(), async_lsp::Error> {
    let file_path = document_uri.to_file_path().map_err(|_| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "URI is not a valid file path")
    })?;

    let workspace = resolve_workspace_for_source_path(&file_path).map_err(|lsp_error| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, lsp_error.to_string())
    })?;

    let mut workspace_file_manager = file_manager_with_stdlib(&workspace.root_dir);
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);

    let parsed_files = parse_diff(&workspace_file_manager, state);

    let diagnostics: Vec<_> = workspace
        .into_iter()
        .flat_map(|package| -> Vec<Diagnostic> {
            let (mut context, crate_id) =
                prepare_package(&workspace_file_manager, &parsed_files, package);

            let file_diagnostics = match check_crate(&mut context, crate_id, false, false) {
                Ok(((), warnings)) => warnings,
                Err(errors_and_warnings) => errors_and_warnings,
            };

            let package_root_dir: String = package.root_dir.as_os_str().to_string_lossy().into();

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
        uri: document_uri,
        version: None,
        diagnostics,
    });

    Ok(())
}

pub(super) fn on_exit(
    _state: &mut LspState,
    _params: (),
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}
