//! NOTE: This is a temporary module until https://github.com/noir-lang/noir/issues/1838 is fixed.
//! This is sectioned off, and currently the default implementation, unless the environment variable NOIR_LSP_NO_HACKS is set.
//! This is mainly so that non-hacky code is not considered dead.
use std::{
    collections::HashMap,
    fs,
    future::{self, Future},
    ops::{self, ControlFlow},
    path::{Path, PathBuf},
};

use async_lsp::{ErrorCode, LanguageClient, ResponseError};
use codespan_reporting::files;
use fm::FileManager;
use lsp_types::{
    CodeLens, CodeLensOptions, CodeLensParams, Command, Diagnostic, DiagnosticSeverity,
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, InitializeParams, InitializeResult,
    InitializedParams, Position, PublishDiagnosticsParams, Range, ServerCapabilities,
    TextDocumentSyncOptions,
};
use noirc_driver::{check_crate, create_local_crate, create_non_local_crate, propagate_dep};
use noirc_errors::{DiagnosticKind, FileDiagnostic};
use noirc_frontend::{
    graph::{CrateGraph, CrateId, CrateName, CrateType},
    hir::Context,
};

// I'm guessing this is here so the `lib.rs` file compiles
use crate::LspState;

const TEST_COMMAND: &str = "nargo.test";
const TEST_CODELENS_TITLE: &str = "▶\u{fe0e} Run Test";

// Handlers
// The handlers for `request` are not `async` because it compiles down to lifetimes that can't be added to
// the router. To return a future that fits the trait, it is easiest wrap your implementations in an `async {}`
// block but you can also use `std::future::ready`.
//
// Additionally, the handlers for `notification` aren't async at all.
//
// They are not attached to the `NargoLspService` struct so they can be unit tested with only `LspState`
// and params passed in.

pub fn on_initialize(
    state: &mut LspState,
    params: InitializeParams,
) -> impl Future<Output = Result<InitializeResult, ResponseError>> {
    if let Some(root_uri) = params.root_uri {
        state.root_path = root_uri.to_file_path().ok();
    }

    async {
        let text_document_sync =
            TextDocumentSyncOptions { save: Some(true.into()), ..Default::default() };

        let code_lens = CodeLensOptions { resolve_provider: Some(false) };

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(text_document_sync.into()),
                code_lens_provider: Some(code_lens),
                // Add capabilities before this spread when adding support for one
                ..Default::default()
            },
            server_info: None,
        })
    }
}

pub fn on_shutdown(
    _state: &mut LspState,
    _params: (),
) -> impl Future<Output = Result<(), ResponseError>> {
    async { Ok(()) }
}

pub fn on_code_lens_request(
    state: &mut LspState,
    params: CodeLensParams,
) -> impl Future<Output = Result<Option<Vec<CodeLens>>, ResponseError>> {
    let actual_path = params.text_document.uri.to_file_path().unwrap();
    let (mut context, crate_id) = match create_context_at_path(&state.root_path, &actual_path) {
        Err(err) => return future::ready(Err(err)),
        Ok(res) => res,
    };

    // We ignore the warnings and errors produced by compilation for producing codelenses
    // because we can still get the test functions even if compilation fails
    let _ = check_crate(&mut context, crate_id, false);

    let fm = &context.file_manager;
    let files = fm.as_simple_files();
    let tests = context.get_all_test_functions_in_crate_matching(&crate_id, "");

    let mut lenses: Vec<CodeLens> = vec![];
    for (func_name, func_id) in tests {
        let location = context.function_meta(&func_id).name.location;
        let file_id = location.file;
        // TODO(#1681): This file_id never be 0 because the "path" where it maps is the directory, not a file
        if file_id.as_usize() != 0 {
            continue;
        }

        let range =
            byte_span_to_range(files, file_id.as_usize(), location.span.into()).unwrap_or_default();

        let command = Command {
            title: TEST_CODELENS_TITLE.into(),
            command: TEST_COMMAND.into(),
            arguments: Some(vec![func_name.into()]),
        };

        let lens = CodeLens { range, command: command.into(), data: None };

        lenses.push(lens);
    }

    let res = if lenses.is_empty() { Ok(None) } else { Ok(Some(lenses)) };

    future::ready(res)
}

pub fn on_initialized(
    _state: &mut LspState,
    _params: InitializedParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

pub fn on_did_change_configuration(
    _state: &mut LspState,
    _params: DidChangeConfigurationParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

pub fn on_did_open_text_document(
    _state: &mut LspState,
    _params: DidOpenTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

pub fn on_did_change_text_document(
    _state: &mut LspState,
    _params: DidChangeTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

pub fn on_did_close_text_document(
    _state: &mut LspState,
    _params: DidCloseTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

/// Find the nearest parent file with given names.
fn find_nearest_parent_file(path: &Path, filenames: &[&str]) -> Option<PathBuf> {
    let mut current_path = path;

    while let Some(parent_path) = current_path.parent() {
        for filename in filenames {
            let mut possible_file_path = parent_path.to_path_buf();
            possible_file_path.push(filename);
            if possible_file_path.is_file() {
                return Some(possible_file_path);
            }
        }
        current_path = parent_path;
    }

    None
}

fn read_dependencies(
    nargo_toml_path: &Path,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let content: String = fs::read_to_string(nargo_toml_path)?;
    let value: toml::Value = toml::from_str(&content)?;

    let mut dependencies = HashMap::new();

    if let Some(toml::Value::Table(table)) = value.get("dependencies") {
        for (key, value) in table {
            if let toml::Value::Table(inner_table) = value {
                if let Some(toml::Value::String(path)) = inner_table.get("path") {
                    dependencies.insert(key.clone(), path.clone());
                }
            }
        }
    }

    Ok(dependencies)
}

pub fn on_did_save_text_document(
    state: &mut LspState,
    params: DidSaveTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    let actual_path = params.text_document.uri.to_file_path().unwrap();
    let actual_file_name = actual_path.file_name();
    let (mut context, crate_id) = match create_context_at_path(&state.root_path, &actual_path) {
        Err(err) => return ControlFlow::Break(Err(err.into())),
        Ok(res) => res,
    };

    let file_diagnostics = match check_crate(&mut context, crate_id, false) {
        Ok(warnings) => warnings,
        Err(errors_and_warnings) => errors_and_warnings,
    };
    let mut diagnostics = Vec::new();

    if !file_diagnostics.is_empty() {
        let fm = &context.file_manager;
        let files = fm.as_simple_files();

        for FileDiagnostic { file_id, diagnostic } in file_diagnostics {
            // TODO(AD): HACK, undo these total hacks once we have a proper approach
            if file_id.as_usize() == 0 {
                // main.nr case
                if actual_file_name.unwrap().to_str() != Some("main.nr")
                    && actual_file_name.unwrap().to_str() != Some("lib.nr")
                {
                    continue;
                }
            } else if fm.path(file_id).file_name().unwrap().to_str().unwrap()
                != actual_file_name.unwrap().to_str().unwrap().replace(".nr", "")
            {
                // every other file case
                continue; // TODO(AD): HACK, we list all errors, filter by hacky final path component
            }

            let mut range = Range::default();

            // TODO: Should this be the first item in secondaries? Should we bail when we find a range?
            for sec in diagnostic.secondaries {
                // Not using `unwrap_or_default` here because we don't want to overwrite a valid range with a default range
                if let Some(r) = byte_span_to_range(files, file_id.as_usize(), sec.span.into()) {
                    range = r
                }
            }
            let severity = match diagnostic.kind {
                DiagnosticKind::Error => Some(DiagnosticSeverity::ERROR),
                DiagnosticKind::Warning => Some(DiagnosticSeverity::WARNING),
            };
            diagnostics.push(Diagnostic {
                range,
                severity,
                message: diagnostic.message,
                ..Diagnostic::default()
            })
        }
    }

    let _ = state.client.publish_diagnostics(PublishDiagnosticsParams {
        uri: params.text_document.uri,
        version: None,
        diagnostics,
    });

    ControlFlow::Continue(())
}

fn create_context_at_path(
    root_path: &Option<PathBuf>,
    actual_path: &Path,
) -> Result<(Context, CrateId), ResponseError> {
    let mut context = match &root_path {
        Some(root_path) => {
            let fm = FileManager::new(root_path);
            let graph = CrateGraph::default();
            Context::new(fm, graph)
        }
        None => {
            let err = ResponseError::new(ErrorCode::REQUEST_FAILED, "Project has not been built");
            return Err(err);
        }
    };

    let mut file_path = actual_path.to_path_buf();
    // TODO better naming/unhacking
    if let Some(new_path) = find_nearest_parent_file(&file_path, &["lib.nr", "main.nr"]) {
        file_path = new_path; // TODO unhack
    }
    let nargo_toml_path = find_nearest_parent_file(&file_path, &["Nargo.toml"]);

    let current_crate_id = create_local_crate(&mut context, &file_path, CrateType::Binary);

    // TODO(AD): undo hacky dependency resolution
    if let Some(nargo_toml_path) = nargo_toml_path {
        let dependencies = read_dependencies(&nargo_toml_path);
        if let Ok(dependencies) = dependencies {
            for (crate_name, dependency_path) in dependencies.iter() {
                let path_to_lib = nargo_toml_path
                    .parent()
                    .unwrap() // TODO
                    .join(PathBuf::from(&dependency_path).join("src").join("lib.nr"));
                let library_crate =
                    create_non_local_crate(&mut context, &path_to_lib, CrateType::Library);
                propagate_dep(&mut context, library_crate, &CrateName::new(crate_name).unwrap());
            }
        }
    }
    Ok((context, current_crate_id))
}

pub fn on_exit(_state: &mut LspState, _params: ()) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

fn byte_span_to_range<'a, F: files::Files<'a> + ?Sized>(
    files: &'a F,
    file_id: F::FileId,
    span: ops::Range<usize>,
) -> Option<Range> {
    // TODO(#1683): Codespan ranges are often (always?) off by some amount of characters
    if let Ok(codespan_range) = codespan_lsp::byte_span_to_range(files, file_id, span) {
        // We have to manually construct a Range because the codespan_lsp restricts lsp-types to the wrong version range
        // TODO: codespan is unmaintained and we should probably subsume it. Ref https://github.com/brendanzab/codespan/issues/345
        let range = Range {
            start: Position {
                line: codespan_range.start.line,
                character: codespan_range.start.character,
            },
            end: Position {
                line: codespan_range.end.line,
                character: codespan_range.end.character,
            },
        };
        Some(range)
    } else {
        None
    }
}
