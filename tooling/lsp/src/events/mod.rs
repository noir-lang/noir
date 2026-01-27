//! Includes various internal events and event handlers so that processing of LSP
//! notifications and requests is done correctly. That is:
//! - notifications should be handled quickly, as they are synchronous and block the main UI thread
//! - requests can be hanled slowly, as they are asynchronous

use std::collections::{HashMap, HashSet};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use fm::{FileId, FileManager};
use nargo::workspace::Workspace;
use noirc_driver::check_crate;
use noirc_frontend::hir::ParsedFiles;

use crate::requests::{
    on_code_action_request_inner, on_completion_request_inner, on_document_symbol_request_inner,
    on_expand_request_inner, on_folding_range_request_inner, on_goto_declaration_request_inner,
    on_goto_definition_request_inner, on_hover_request_inner, on_inlay_hint_request_inner,
    on_prepare_rename_request_inner, on_references_request_inner, on_rename_request_inner,
    on_semantic_tokens_full_request_inner, on_signature_help_request_inner,
    on_workspace_symbol_request_inner, uri_to_file_path,
};
use crate::{
    LspState, PackageCacheData, get_package_tests_in_crate,
    types::{NargoPackageTests, notification},
};
use crate::{PendingRequest, WorkspaceCacheData, parse_diff};
use async_lsp::LanguageClient;
use async_lsp::lsp_types;
use async_lsp::lsp_types::{DiagnosticRelatedInformation, DiagnosticTag, Url};
use fm::FileMap;
use noirc_errors::reporter::CustomLabel;
use noirc_errors::{CustomDiagnostic, DiagnosticKind, Location};

use crate::types::{Diagnostic, DiagnosticSeverity, PublishDiagnosticsParams};

use crate::byte_span_to_range;
use crate::requests::file_path_to_file_id;
use fm::PathString;
use noirc_frontend::elaborator::{FrontendOptions, UnstableFeature};
use noirc_frontend::hir::Context;
use noirc_frontend::hir::def_collector::dc_crate::DefCollector;
use noirc_frontend::hir::def_map::{CrateDefMap, LocalModuleId};
use noirc_frontend::parse_program;

/// An event to type-check the entire workspace.
pub(crate) struct ProcessWorkspaceEvent {
    pub(crate) workspace: Workspace,
    pub(crate) file_manager: FileManager,
    pub(crate) parsed_files: ParsedFiles,
}

pub(crate) fn on_process_workspace_event(
    state: &mut LspState,
    event: ProcessWorkspaceEvent,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    let workspace = event.workspace;
    let file_manager = event.file_manager;
    let parsed_files = event.parsed_files;

    for package in workspace.into_iter() {
        let (mut context, crate_id) = crate::prepare_package(&file_manager, &parsed_files, package);

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

    state.workspace_cache.insert(workspace.root_dir.clone(), WorkspaceCacheData { file_manager });

    finish_type_checking(state);

    ControlFlow::Continue(())
}

/// An event to type-check only a single file that has changed.
pub(crate) struct ProcessWorkspaceForSingleFileChangeEvent {
    pub(crate) workspace: Workspace,
    pub(crate) file_uri: Url,
    pub(crate) file_source: String,
}

pub(crate) fn on_process_workspace_for_single_file_change(
    state: &mut LspState,
    event: ProcessWorkspaceForSingleFileChangeEvent,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    let workspace = event.workspace;
    let file_uri = event.file_uri;
    let file_source = &event.file_source;

    let root_dir = &workspace.root_dir;
    let mut workspace_cache = state.workspace_cache.remove(root_dir).unwrap();
    let mut package_cache = state.package_cache.remove(root_dir).unwrap();

    let mut file_manager = workspace_cache.file_manager;
    let file_map = file_manager.as_file_map();
    let file_path = match uri_to_file_path(&file_uri) {
        Ok(file_path) => file_path,
        Err(err) => {
            finish_type_checking(state);
            return ControlFlow::Break(Err(async_lsp::Error::Response(err)));
        }
    };

    // We need to replace the file's source in the file manager
    let file_id = match file_path_to_file_id(file_map, &PathString::from(&file_path)) {
        Ok(file_id) => file_id,
        Err(err) => {
            finish_type_checking(state);
            return ControlFlow::Break(Err(async_lsp::Error::Response(err)));
        }
    };
    file_manager.replace_file(file_id, file_source.to_string());

    let mut node_interner = package_cache.node_interner;

    // Clear some locations associated with this file. For example, locations associated
    // with `ExprId` will be cleared. These lookups are used everywhere by LSP.
    // This also removes methods that were defined in this file.
    node_interner.clear_in_file(file_id);

    let mut def_maps = package_cache.def_maps;
    let crate_graph = package_cache.crate_graph;

    let crate_id = package_cache.crate_id;

    // Get a hold of the CrateDefMap for this crate, by removing it.
    // There's no need to explicitly add it back: DefCollector::collect_defs_and_elaborate will do it.
    let mut crate_def_map = def_maps.remove(&crate_id).unwrap();

    // Find out the local module ID corresponding to this file
    let module_index = crate_def_map
        .modules()
        .iter()
        .find(|(_, module_data)| module_data.location.file == file_id)
        .unwrap()
        .0;
    let module_id = LocalModuleId::new(module_index);

    // Clear all the definitions in the existing module (and its children, as long as they happen in
    // the same file), as they shouldn't be offered in autocompletion, references, etc., anymore.
    clear_all_in_file(file_id, module_id, &mut crate_def_map);

    // Parse the program and add it to the parsed files
    let (parsed_program, errors) = parse_program(file_source, file_id);
    let mut parsed_files = parse_diff(&file_manager, state);
    parsed_files.insert(file_id, (parsed_program.clone(), errors));

    // Prepare some things to create a Context
    let sorted_module = parsed_program.into_sorted();
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
    let reuse_existing_module_declarations = true;
    let mut errors = Vec::new();
    DefCollector::collect_defs_and_elaborate(
        sorted_module,
        file_id,
        module_id,
        crate_id,
        &mut context,
        def_collector,
        options,
        reuse_existing_module_declarations,
        &mut errors,
    );

    // Put some things back before restoring the cached data
    package_cache.node_interner = context.def_interner;
    package_cache.def_maps = context.def_maps;
    package_cache.crate_graph = context.crate_graph;

    workspace_cache.file_manager = file_manager;

    state.workspace_cache.insert(root_dir.clone(), workspace_cache);
    state.package_cache.insert(root_dir.clone(), package_cache);

    finish_type_checking(state);

    ControlFlow::Continue(())
}

fn finish_type_checking(state: &mut LspState) {
    state.pending_type_check_events -= 1;
    if state.pending_type_check_events == 0 {
        let _ = state.client.emit(ProcessRequestQueueEvent);
    }
}

/// An event to process all pending requests in the queue.
/// This event is triggered when a type-checking operation finished and there are no
/// more type-checking operations to do.
pub(crate) struct ProcessRequestQueueEvent;

pub(crate) fn on_process_request_queue_event(
    state: &mut LspState,
    _event: ProcessRequestQueueEvent,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    for request in std::mem::take(&mut state.pending_requests) {
        match request {
            PendingRequest::Completion { params, tx } => {
                let result = on_completion_request_inner(state, params);
                let _ = tx.send(result);
            }
            PendingRequest::CodeAction { params, tx } => {
                let result = on_code_action_request_inner(state, params);
                let _ = tx.send(result);
            }
            PendingRequest::DocumentSymbol { params, tx } => {
                let result = on_document_symbol_request_inner(state, params);
                let _ = tx.send(result);
            }
            PendingRequest::InlayHint { params, tx } => {
                let result = on_inlay_hint_request_inner(state, params);
                let _ = tx.send(result);
            }
            PendingRequest::Expand { params, tx } => {
                let result = on_expand_request_inner(state, params);
                let _ = tx.send(result);
            }
            PendingRequest::FoldingRange { params, tx } => {
                let result = on_folding_range_request_inner(state, params);
                let _ = tx.send(result);
            }
            PendingRequest::GotoDeclaration { params, tx } => {
                let result = on_goto_declaration_request_inner(state, params);
                let _ = tx.send(result);
            }
            PendingRequest::GotoDefinition { params, return_type_location_instead, tx } => {
                let result =
                    on_goto_definition_request_inner(state, params, return_type_location_instead);
                let _ = tx.send(result);
            }
            PendingRequest::Hover { params, tx } => {
                let result = on_hover_request_inner(state, params);
                let _ = tx.send(result);
            }
            PendingRequest::References { params, tx } => {
                let result = on_references_request_inner(state, params);
                let _ = tx.send(result);
            }
            PendingRequest::PrepareRename { params, tx } => {
                let result = on_prepare_rename_request_inner(state, params);
                let _ = tx.send(result);
            }
            PendingRequest::Rename { params, tx } => {
                let result = on_rename_request_inner(state, params);
                let _ = tx.send(result);
            }
            PendingRequest::SemanticTokens { params, tx } => {
                let result = on_semantic_tokens_full_request_inner(state, params);
                let _ = tx.send(result);
            }
            PendingRequest::SignatureHelp { params, tx } => {
                let result = on_signature_help_request_inner(state, params);
                let _ = tx.send(result);
            }
            PendingRequest::WorkspaceSymbol { params, tx } => {
                let result = on_workspace_symbol_request_inner(state, params);
                let _ = tx.send(result);
            }
        }
    }

    ControlFlow::Continue(())
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

fn clear_all_in_file(file: FileId, module_id: LocalModuleId, crate_def_map: &mut CrateDefMap) {
    let mut module_ids = vec![module_id];

    while let Some(module_id) = module_ids.pop() {
        let module_data = &mut crate_def_map[module_id];
        if module_data.location.file != file {
            continue;
        }

        module_data.clear();

        for child_module_id in module_data.children.values() {
            module_ids.push(*child_module_id);
        }
    }
}
