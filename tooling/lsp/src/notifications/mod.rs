use std::collections::HashMap;
use std::collections::{BTreeMap, HashSet};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::str::FromStr as _;

use crate::requests::{file_path_to_file_id, uri_to_file_path};
use crate::{
    PackageCacheData, WorkspaceCacheData, insert_all_files_for_workspace_into_file_manager,
};
use async_lsp::lsp_types;
use async_lsp::lsp_types::{DiagnosticRelatedInformation, DiagnosticTag, Url};
use async_lsp::{ErrorCode, LanguageClient, ResponseError};
use fm::{FileManager, FileMap, PathString};
use nargo::package::{Package, PackageType};
use nargo::workspace::Workspace;
use noirc_driver::check_crate;
use noirc_driver::{CrateName, NOIR_ARTIFACT_VERSION_STRING};
use noirc_errors::reporter::CustomLabel;
use noirc_errors::{CustomDiagnostic, DiagnosticKind, Location};
use noirc_frontend::elaborator::{FrontendOptions, UnstableFeature};
use noirc_frontend::hir::Context;
use noirc_frontend::hir::def_collector::dc_crate::DefCollector;
use noirc_frontend::hir::def_map::LocalModuleId;
use noirc_frontend::parse_program;

use crate::types::{
    Diagnostic, DiagnosticSeverity, DidChangeConfigurationParams, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    InitializedParams, NargoPackageTests, PublishDiagnosticsParams, notification,
};

use crate::{
    LspState, byte_span_to_range, get_package_tests_in_crate, parse_diff,
    resolve_workspace_for_source_path,
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

    match process_workspace(state, &workspace) {
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
        process_workspace(state, &workspace)
    }
}

fn handle_on_did_change_text_document_notification(
    state: &mut LspState,
    document_uri: Url,
    text: &str,
) -> Result<(), async_lsp::Error> {
    let workspace = workspace_from_document_uri(document_uri.clone())?;

    if state.package_cache.contains_key(&workspace.root_dir) {
        process_workspace_for_single_file_change(state, &workspace, document_uri, text)
    } else {
        // If it's the first time we see this package, show diagnostics.
        // This can happen for example when a user opens a Noir file in a package for the first time.
        process_workspace(state, &workspace)
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
    workspace: &Workspace,
) -> Result<(), async_lsp::Error> {
    let mut workspace_file_manager = workspace.new_file_manager();
    if workspace.is_assumed {
        let package = workspace.members.first().unwrap();
        workspace_file_manager
            .add_file_with_source_canonical_path(&package.entry_path, String::new());
    } else {
        insert_all_files_for_workspace_into_file_manager(
            state,
            workspace,
            &mut workspace_file_manager,
        );
    }

    let parsed_files = parse_diff(&workspace_file_manager, state);

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

        state.package_cache.insert(
            package.root_dir.clone(),
            PackageCacheData {
                crate_id,
                crate_graph: context.crate_graph,
                node_interner: context.def_interner,
                def_maps: context.def_maps,
                usage_tracker: context.usage_tracker,
            },
        );

        let fm = &context.file_manager;

        publish_diagnostics(state, &package.root_dir, fm, file_diagnostics);
    }

    state.workspace_cache.insert(
        workspace.root_dir.clone(),
        WorkspaceCacheData { file_manager: workspace_file_manager },
    );

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
    workspace: &Workspace,
    file_uri: Url,
    file_source: &str,
) -> Result<(), async_lsp::Error> {
    let root_dir = &workspace.root_dir;
    let mut workspace_cache = state.workspace_cache.remove(root_dir).unwrap();
    let mut package_cache = state.package_cache.remove(root_dir).unwrap();

    let mut file_manager = workspace_cache.file_manager;
    let file_map = file_manager.as_file_map();
    let file_path = uri_to_file_path(&file_uri)?;

    // We need to replace the file's source in the file manager
    let file_id = file_path_to_file_id(file_map, &PathString::from(&file_path))?;
    file_manager.replace_file(file_id, file_source.to_string());

    // Clear some locations associated with this file. For example, locations associated
    // with `ExprId` will be cleared. These lookups are used everywhere by LSP.
    let mut node_interner = package_cache.node_interner;
    node_interner.clear_file_locations(file_id);

    let mut def_maps = package_cache.def_maps;
    let crate_graph = package_cache.crate_graph;

    let crate_id = package_cache.crate_id;

    // Get a hold of the CrateDefMap for this crate, by removing it.
    // There's no need to explicitly add it back: DefCollector::collect_defs_and_elaborate will do it.
    let crate_def_map = def_maps.remove(&crate_id).unwrap();

    // Parse the program and add it to the parsed files
    let (parsed_program, errors) = parse_program(file_source, file_id);
    let mut parsed_files = parse_diff(&file_manager, state);
    parsed_files.insert(file_id, (parsed_program.clone(), errors));

    // Prepare some things to create a Context
    let sorted_module = parsed_program.into_sorted();
    let module_index = crate_def_map
        .modules()
        .iter()
        .find(|(_, module_data)| module_data.location.file == file_id)
        .unwrap()
        .0;
    let module_id = LocalModuleId::new(module_index);
    let def_collector = DefCollector::new(crate_def_map);
    let mut context =
        Context::from_existing(&file_manager, &parsed_files, node_interner, def_maps, crate_graph);

    // Here we enable all options because we won't show errors to users, so it's easier to
    // assume all unstable features are enabled.
    let options = FrontendOptions {
        debug_comptime_in_file: None,
        enabled_unstable_features: &[
            UnstableFeature::Enums,
            UnstableFeature::Ownership,
            UnstableFeature::TraitAsType,
        ],
        disable_required_unstable_features: false,
    };

    // This is when the type-checking of this single file happens
    let shallow = true;
    let mut errors = Vec::new();
    DefCollector::collect_defs_and_elaborate(
        sorted_module,
        file_id,
        module_id,
        crate_id,
        &mut context,
        def_collector,
        options,
        shallow,
        &mut errors,
    );

    // Put some things back before restoring the cached data
    package_cache.node_interner = context.def_interner;
    package_cache.def_maps = context.def_maps;
    package_cache.crate_graph = context.crate_graph;

    workspace_cache.file_manager = file_manager;

    state.workspace_cache.insert(root_dir.clone(), workspace_cache);
    state.package_cache.insert(root_dir.clone(), package_cache);

    Ok(())
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

fn publish_diagnostics(
    state: &mut LspState,
    package_root_dir: &PathBuf,
    fm: &FileManager,
    custom_diagnostics: Vec<CustomDiagnostic>,
) {
    let files = fm.as_file_map();
    let mut diagnostics_per_url: HashMap<Url, Vec<Diagnostic>> = HashMap::default();

    for custom_diagnostic in custom_diagnostics.into_iter() {
        let file = custom_diagnostic.file;
        let path = fm.path(file).expect("file must exist to have emitted diagnostic");
        if let Some(uri) = uri_from_path(path) {
            if let Some(diagnostic) =
                custom_diagnostic_to_diagnostic(custom_diagnostic, files, fm, uri.clone())
            {
                diagnostics_per_url.entry(uri).or_default().push(diagnostic);
            }
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

fn custom_diagnostic_to_diagnostic(
    diagnostic: CustomDiagnostic,
    files: &FileMap,
    fm: &FileManager,
    uri: Url,
) -> Option<Diagnostic> {
    if diagnostic.secondaries.is_empty() {
        return None;
    }

    let span = diagnostic.secondaries.first().unwrap().location.span;
    let range = byte_span_to_range(files, diagnostic.file, span.into())?;

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

    let secondaries = diagnostic
        .secondaries
        .into_iter()
        .filter_map(|secondary| secondary_to_related_information(secondary, files, fm));
    let notes = diagnostic.notes.into_iter().map(|message| DiagnosticRelatedInformation {
        location: lsp_types::Location { uri: uri.clone(), range },
        message,
    });
    let call_stack = diagnostic
        .call_stack
        .into_iter()
        .rev()
        .filter_map(|frame| call_stack_frame_to_related_information(frame, files, fm));
    let related_information: Vec<_> = secondaries.chain(notes).chain(call_stack).collect();

    Some(Diagnostic {
        range,
        severity: Some(severity),
        message: diagnostic.message,
        tags: if tags.is_empty() { None } else { Some(tags) },
        related_information: if related_information.is_empty() {
            None
        } else {
            Some(related_information)
        },
        ..Default::default()
    })
}

fn secondary_to_related_information(
    secondary: CustomLabel,
    files: &FileMap,
    fm: &FileManager,
) -> Option<DiagnosticRelatedInformation> {
    let secondary_file = secondary.location.file;
    let path = fm.path(secondary_file)?;
    let uri = uri_from_path(path)?;
    let range = byte_span_to_range(files, secondary_file, secondary.location.span.into())?;
    let message = secondary.message;
    Some(DiagnosticRelatedInformation { location: lsp_types::Location { uri, range }, message })
}

fn uri_from_path(path: &Path) -> Option<Url> {
    if let Ok(uri) = Url::from_file_path(path) {
        Some(uri)
    } else if path.starts_with("std") {
        Some(Url::parse(&format!("noir-std://{}", path.to_string_lossy())).unwrap())
    } else {
        None
    }
}

fn call_stack_frame_to_related_information(
    frame: Location,
    files: &FileMap,
    fm: &FileManager,
) -> Option<DiagnosticRelatedInformation> {
    let path = fm.path(frame.file)?;
    let uri = uri_from_path(path)?;
    let range = byte_span_to_range(files, frame.file, frame.span.into())?;
    Some(DiagnosticRelatedInformation {
        location: lsp_types::Location { uri, range },
        message: "Error originated here".to_string(),
    })
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
