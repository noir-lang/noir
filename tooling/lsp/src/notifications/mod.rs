use std::collections::HashSet;
use std::ops::ControlFlow;
use std::path::PathBuf;

use crate::insert_all_files_for_workspace_into_file_manager;
use async_lsp::{ErrorCode, LanguageClient, ResponseError};
use fm::{FileId, FileManager, FileMap};
use fxhash::FxHashMap as HashMap;
use lsp_types::{DiagnosticTag, Url};
use noirc_driver::check_crate;
use noirc_errors::{DiagnosticKind, FileDiagnostic};
use noirc_frontend::ast::{NoirFunction, NoirTraitImpl, TraitImplItemKind, TypeImpl};
use noirc_frontend::parser::ItemKind;
use noirc_frontend::ParsedModule;

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

pub(crate) fn on_did_open_text_document(
    state: &mut LspState,
    params: DidOpenTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    state.input_files.insert(params.text_document.uri.to_string(), params.text_document.text);

    let document_uri = params.text_document.uri;
    let output_diagnostics = true;
    let only_check_open_files = false;

    match process_workspace_for_noir_document(
        state,
        document_uri,
        output_diagnostics,
        only_check_open_files,
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
    let output_diagnostics = false;
    let only_check_open_files = true;

    match process_workspace_for_noir_document(
        state,
        document_uri,
        output_diagnostics,
        only_check_open_files,
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
    let output_diagnostics = false;
    let only_check_open_files = false;

    match process_workspace_for_noir_document(
        state,
        document_uri,
        output_diagnostics,
        only_check_open_files,
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
    let output_diagnostics = true;
    let only_check_open_files = false;

    match process_workspace_for_noir_document(
        state,
        document_uri,
        output_diagnostics,
        only_check_open_files,
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
    document_uri: Url,
    output_diagnostics: bool,
    only_check_open_files: bool,
) -> Result<(), async_lsp::Error> {
    let file_path = document_uri.to_file_path().map_err(|_| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "URI is not a valid file path")
    })?;

    let workspace = resolve_workspace_for_source_path(&file_path).map_err(|lsp_error| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, lsp_error.to_string())
    })?;

    let mut workspace_file_manager = workspace.new_file_manager();

    insert_all_files_for_workspace_into_file_manager(
        state,
        &workspace,
        &mut workspace_file_manager,
    );

    let mut parsed_files = parse_diff(&workspace_file_manager, state);

    // If we only want to check the currently open files, we empty function bodies of files that aren't open.
    // These function bodies will error, but we are not interested in those errors (we don't report errors in this
    // case). The function types are correct, though, because the types in the signature remain unchanged.
    // Doing this greatly speeds up the time it takes to reanalyze a crate after incremental edits without saving
    // (type-checking body functions is much slower than emptying them)
    if only_check_open_files {
        let mut currently_open_files: HashSet<FileId> = HashSet::new();
        for filename in state.input_files.keys() {
            let filename = filename.strip_prefix("file://").unwrap();
            if let Some(file_id) = workspace_file_manager.name_to_id(PathBuf::from(filename)) {
                currently_open_files.insert(file_id);
            }
        }

        for (file_id, (parsed_module, _errors)) in parsed_files.iter_mut() {
            if !currently_open_files.is_empty() && !currently_open_files.contains(file_id) {
                empty_parsed_module_function_bodies(parsed_module);
            }
        }
    };

    for package in workspace.into_iter() {
        let (mut context, crate_id) =
            crate::prepare_package(&workspace_file_manager, &parsed_files, package);

        let file_diagnostics = match check_crate(&mut context, crate_id, &Default::default()) {
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
        state.cached_definitions.insert(package.root_dir.clone(), context.def_interner);
        state.cached_def_maps.insert(package.root_dir.clone(), context.def_maps);

        let fm = &context.file_manager;
        let files = fm.as_file_map();

        if output_diagnostics {
            publish_diagnostics(state, &package.root_dir, files, fm, file_diagnostics);
        }
    }

    Ok(())
}

fn publish_diagnostics(
    state: &mut LspState,
    package_root_dir: &PathBuf,
    files: &FileMap,
    fm: &FileManager,
    file_diagnostics: Vec<FileDiagnostic>,
) {
    let mut diagnostics_per_url: HashMap<Url, Vec<Diagnostic>> = HashMap::default();

    for file_diagnostic in file_diagnostics.into_iter() {
        let file_id = file_diagnostic.file_id;
        let diagnostic = file_diagnostic_to_diagnostic(file_diagnostic, files);

        let path = fm.path(file_id).expect("file must exist to have emitted diagnostic");
        if let Ok(uri) = Url::from_file_path(path) {
            diagnostics_per_url.entry(uri).or_default().push(diagnostic);
        }
    }

    let new_files_with_errors: HashSet<_> = diagnostics_per_url.keys().cloned().collect();

    for (uri, diagnostics) in diagnostics_per_url {
        let _ = state.client.publish_diagnostics(PublishDiagnosticsParams {
            uri,
            version: None,
            diagnostics,
        });
    }

    // For files that previously had errors but no longer have errors we still need to publish empty diagnostics
    if let Some(old_files_with_errors) = state.files_with_errors.get(package_root_dir) {
        for uri in old_files_with_errors.difference(&new_files_with_errors) {
            let _ = state.client.publish_diagnostics(PublishDiagnosticsParams {
                uri: uri.clone(),
                version: None,
                diagnostics: vec![],
            });
        }
    }

    // Remember which files currently have errors, for next time
    state.files_with_errors.insert(package_root_dir.clone(), new_files_with_errors);
}

fn file_diagnostic_to_diagnostic(file_diagnostic: FileDiagnostic, files: &FileMap) -> Diagnostic {
    let file_id = file_diagnostic.file_id;
    let diagnostic = file_diagnostic.diagnostic;

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

    let mut tags = Vec::new();
    if diagnostic.unnecessary {
        tags.push(DiagnosticTag::UNNECESSARY);
    }
    if diagnostic.deprecated {
        tags.push(DiagnosticTag::DEPRECATED);
    }

    Diagnostic {
        range,
        severity: Some(severity),
        message: diagnostic.message,
        tags: if tags.is_empty() { None } else { Some(tags) },
        ..Default::default()
    }
}

pub(super) fn on_exit(
    _state: &mut LspState,
    _params: (),
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

fn empty_parsed_module_function_bodies(parsed_module: &mut ParsedModule) {
    for item in &mut parsed_module.items {
        match &mut item.kind {
            ItemKind::Function(noir_function) => empty_noir_function_body(noir_function),
            ItemKind::TraitImpl(noir_trait_impl) => {
                empty_noir_trait_impl_function_bodies(noir_trait_impl)
            }
            ItemKind::Impl(noir_impl) => empty_noir_impl_function_bodies(noir_impl),
            ItemKind::Submodules(parsed_sub_module) => {
                empty_parsed_module_function_bodies(&mut parsed_sub_module.contents)
            }
            ItemKind::Import(_, _)
            | ItemKind::Struct(_)
            | ItemKind::Trait(_)
            | ItemKind::TypeAlias(_)
            | ItemKind::Global(_)
            | ItemKind::ModuleDecl(_)
            | ItemKind::InnerAttribute(_) => (),
        }
    }
}

fn empty_noir_trait_impl_function_bodies(noir_trait_impl: &mut NoirTraitImpl) {
    for item in &mut noir_trait_impl.items {
        match &mut item.item.kind {
            TraitImplItemKind::Function(noir_function) => empty_noir_function_body(noir_function),
            TraitImplItemKind::Constant(..) | TraitImplItemKind::Type { .. } => (),
        }
    }
}

fn empty_noir_impl_function_bodies(noir_impl: &mut TypeImpl) {
    for (noir_function, _span) in &mut noir_impl.methods {
        empty_noir_function_body(&mut noir_function.item)
    }
}

fn empty_noir_function_body(noir_function: &mut NoirFunction) {
    noir_function.def.body.statements.clear()
}

#[cfg(test)]
mod notification_tests {
    use crate::test_utils;

    use super::*;
    use lsp_types::{
        InlayHintLabel, InlayHintParams, Position, Range, TextDocumentContentChangeEvent,
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
