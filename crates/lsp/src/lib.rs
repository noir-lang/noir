use std::{
    collections::HashMap,
    fs,
    future::Future,
    ops::{self, ControlFlow},
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};

use acvm::Language;
use async_lsp::{
    router::Router, AnyEvent, AnyNotification, AnyRequest, ClientSocket, Error, LanguageClient,
    LspService, ResponseError,
};
use codespan_reporting::files;
use lsp_types::{
    notification, request, CodeLens, CodeLensOptions, CodeLensParams, Command, Diagnostic,
    DiagnosticSeverity, DidChangeConfigurationParams, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    InitializeParams, InitializeResult, InitializedParams, Position, PublishDiagnosticsParams,
    Range, ServerCapabilities, TextDocumentSyncOptions,
};
use noirc_driver::Driver;
use noirc_errors::{DiagnosticKind, FileDiagnostic};
use noirc_frontend::graph::{CrateName, CrateType};
use serde_json::Value as JsonValue;
use tower::Service;

const TEST_COMMAND: &str = "nargo.test";
const TEST_CODELENS_TITLE: &str = "▶\u{fe0e} Run Test";

// State for the LSP gets implemented on this struct and is internal to the implementation
#[derive(Debug)]
struct LspState {
    client: ClientSocket,
}

impl LspState {
    fn new(client: &ClientSocket) -> Self {
        Self { client: client.clone() }
    }
}

pub struct NargoLspService {
    router: Router<LspState>,
}

impl NargoLspService {
    pub fn new(client: &ClientSocket) -> Self {
        let state = LspState::new(client);
        let mut router = Router::new(state);
        router
            .request::<request::Initialize, _>(on_initialize)
            .request::<request::Shutdown, _>(on_shutdown)
            .request::<request::CodeLensRequest, _>(on_code_lens_request)
            .notification::<notification::Initialized>(on_initialized)
            .notification::<notification::DidChangeConfiguration>(on_did_change_configuration)
            .notification::<notification::DidOpenTextDocument>(on_did_open_text_document)
            .notification::<notification::DidChangeTextDocument>(on_did_change_text_document)
            .notification::<notification::DidCloseTextDocument>(on_did_close_text_document)
            .notification::<notification::DidSaveTextDocument>(on_did_save_text_document)
            .notification::<notification::Exit>(on_exit);
        Self { router }
    }
}

// This trait implemented as a passthrough to the router, which makes
// our `NargoLspService` a normal Service as far as Tower is concerned.
impl Service<AnyRequest> for NargoLspService {
    type Response = JsonValue;
    type Error = ResponseError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.router.poll_ready(cx)
    }

    fn call(&mut self, req: AnyRequest) -> Self::Future {
        self.router.call(req)
    }
}

// This trait implemented as a passthrough to the router, which makes
// our `NargoLspService` able to accept the `async-lsp` middleware.
impl LspService for NargoLspService {
    fn notify(&mut self, notification: AnyNotification) -> ControlFlow<Result<(), Error>> {
        self.router.notify(notification)
    }

    fn emit(&mut self, event: AnyEvent) -> ControlFlow<Result<(), Error>> {
        self.router.emit(event)
    }
}

// Handlers
// The handlers for `request` are not `async` because it compiles down to lifetimes that can't be added to
// the router. To return a future that fits the trait, it is easiest wrap your implementations in an `async {}`
// block but you can also use `std::future::ready`.
//
// Additionally, the handlers for `notification` aren't async at all.
//
// They are not attached to the `NargoLspService` struct so they can be unit tested with only `LspState`
// and params passed in.

fn on_initialize(
    _state: &mut LspState,
    _params: InitializeParams,
) -> impl Future<Output = Result<InitializeResult, ResponseError>> {
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

fn on_shutdown(
    _state: &mut LspState,
    _params: (),
) -> impl Future<Output = Result<(), ResponseError>> {
    async { Ok(()) }
}

fn on_code_lens_request(
    _state: &mut LspState,
    params: CodeLensParams,
) -> impl Future<Output = Result<Option<Vec<CodeLens>>, ResponseError>> {
    async move {
        let actual_path = params.text_document.uri.to_file_path().unwrap();
        let mut driver = create_driver_at_path(actual_path);

        // We ignore the warnings and errors produced by compilation for producing codelenses
        // because we can still get the test functions even if compilation fails
        let _ = driver.check_crate(false);

        let fm = driver.file_manager();
        let files = fm.as_simple_files();
        let tests = driver.get_all_test_functions_in_crate_matching("");

        let mut lenses: Vec<CodeLens> = vec![];
        for func_id in tests {
            let location = driver.function_meta(&func_id).name.location;
            let file_id = location.file;
            // TODO(#1681): This file_id never be 0 because the "path" where it maps is the directory, not a file
            if file_id.as_usize() != 0 {
                continue;
            }

            let func_name = driver.function_name(func_id);

            let range = byte_span_to_range(files, file_id.as_usize(), location.span.into())
                .unwrap_or_default();

            let command = Command {
                title: TEST_CODELENS_TITLE.into(),
                command: TEST_COMMAND.into(),
                arguments: Some(vec![func_name.into()]),
            };

            let lens = CodeLens { range, command: command.into(), data: None };

            lenses.push(lens);
        }

        if lenses.is_empty() {
            Ok(None)
        } else {
            Ok(Some(lenses))
        }
    }
}

fn on_initialized(
    _state: &mut LspState,
    _params: InitializedParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

fn on_did_change_configuration(
    _state: &mut LspState,
    _params: DidChangeConfigurationParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

fn on_did_open_text_document(
    _state: &mut LspState,
    _params: DidOpenTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

fn on_did_change_text_document(
    _state: &mut LspState,
    _params: DidChangeTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    ControlFlow::Continue(())
}

fn on_did_close_text_document(
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

fn on_did_save_text_document(
    state: &mut LspState,
    params: DidSaveTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    let actual_path = params.text_document.uri.to_file_path().unwrap();
    let mut driver = create_driver_at_path(actual_path.clone());

    let file_diagnostics = match driver.check_crate(false) {
        Ok(warnings) => warnings,
        Err(errors_and_warnings) => errors_and_warnings,
    };
    let mut diagnostics = Vec::new();

    if !file_diagnostics.is_empty() {
        let fm = driver.file_manager();
        let files = fm.as_simple_files();

        for FileDiagnostic { file_id, diagnostic } in file_diagnostics {
            // TODO(AD): HACK, undo these total hacks once we have a proper approach
            if file_id.as_usize() == 0 {
                // main.nr case
                if actual_path.file_name().unwrap().to_str() != Some("main.nr")
                    && actual_path.file_name().unwrap().to_str() != Some("lib.nr")
                {
                    continue;
                }
            } else if fm.path(file_id).file_name().unwrap().to_str().unwrap()
                != actual_path.file_name().unwrap().to_str().unwrap().replace(".nr", "")
            {
                if actual_path.file_name().unwrap().to_str() == Some("main.nr")
                    || actual_path.file_name().unwrap().to_str() == Some("lib.nr")
                {
                    continue;
                }
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

fn create_driver_at_path(actual_path: PathBuf) -> Driver {
    // TODO: Requiring `Language` and `is_opcode_supported` to construct a driver makes for some real stinky code
    // The driver should not require knowledge of the backend; instead should be implemented as an independent pass (in nargo?)
    let mut driver = Driver::new(&Language::R1CS, Box::new(|_op| false));

    let mut file_path: PathBuf = actual_path;
    // TODO better naming/unhacking
    if let Some(new_path) = find_nearest_parent_file(&file_path, &["lib.nr", "main.nr"]) {
        file_path = new_path; // TODO unhack
    }
    let nargo_toml_path = find_nearest_parent_file(&file_path, &["Nargo.toml"]);

    driver.create_local_crate(file_path, CrateType::Binary);

    // TODO(AD): undo hacky dependency resolution
    if let Some(nargo_toml_path) = nargo_toml_path {
        let dependencies = read_dependencies(&nargo_toml_path);
        if let Ok(dependencies) = dependencies {
            for (crate_name, dependency_path) in dependencies.iter() {
                let path_to_lib = nargo_toml_path
                    .parent()
                    .unwrap() // TODO
                    .join(PathBuf::from(&dependency_path).join("src").join("lib.nr"));
                let library_crate = driver.create_non_local_crate(path_to_lib, CrateType::Library);
                driver.propagate_dep(library_crate, &CrateName::new(crate_name).unwrap());
            }
        }
    }
    driver
}

fn on_exit(_state: &mut LspState, _params: ()) -> ControlFlow<Result<(), async_lsp::Error>> {
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

#[cfg(test)]
mod lsp_tests {
    use lsp_types::TextDocumentSyncCapability;
    use tokio::test;

    use super::*;

    #[test]
    async fn test_on_initialize() {
        // Not available in published release yet
        let client = ClientSocket::new_closed();
        let mut state = LspState::new(&client);
        let params = InitializeParams::default();
        let response = on_initialize(&mut state, params).await.unwrap();
        assert!(matches!(
            response.capabilities,
            ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions { save: Some(_), .. }
                )),
                code_lens_provider: Some(CodeLensOptions { resolve_provider: Some(false) }),
                ..
            }
        ));
        assert!(response.server_info.is_none());
    }
}
