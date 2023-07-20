mod lib_hacky;
use std::env;

use std::{
    future::{self, Future},
    ops::{self, ControlFlow},
    path::PathBuf,
    pin::Pin,
    task::{self, Poll},
};

use async_lsp::{
    router::Router, AnyEvent, AnyNotification, AnyRequest, ClientSocket, Error, ErrorCode,
    LanguageClient, LspService, ResponseError,
};
use codespan_reporting::files;
use fm::FileManager;
use lsp_types::{
    notification, request, CodeLens, CodeLensOptions, CodeLensParams, Command, Diagnostic,
    DiagnosticSeverity, DidChangeConfigurationParams, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    InitializeParams, InitializeResult, InitializedParams, Position, PublishDiagnosticsParams,
    Range, ServerCapabilities, TextDocumentSyncOptions,
};
use noirc_driver::{check_crate, create_local_crate};
use noirc_errors::{DiagnosticKind, FileDiagnostic};
use noirc_frontend::{
    graph::{CrateGraph, CrateType},
    hir::Context,
};
use serde_json::Value as JsonValue;
use tower::Service;

const TEST_COMMAND: &str = "nargo.test";
const TEST_CODELENS_TITLE: &str = "▶\u{fe0e} Run Test";

// State for the LSP gets implemented on this struct and is internal to the implementation
pub struct LspState {
    root_path: Option<PathBuf>,
    client: ClientSocket,
}

impl LspState {
    fn new(client: &ClientSocket) -> Self {
        Self { client: client.clone(), root_path: None }
    }
}

pub struct NargoLspService {
    router: Router<LspState>,
}

impl NargoLspService {
    pub fn new(client: &ClientSocket) -> Self {
        // Using conditional running with lib_hacky to prevent non-hacky code from being identified as dead code
        // Secondarily, provides a runtime way to stress the non-hacky code.
        if env::var("NOIR_LSP_NO_HACK").is_err() {
            let state = LspState::new(client);
            let mut router = Router::new(state);
            router
                .request::<request::Initialize, _>(lib_hacky::on_initialize)
                .request::<request::Shutdown, _>(lib_hacky::on_shutdown)
                .request::<request::CodeLensRequest, _>(lib_hacky::on_code_lens_request)
                .notification::<notification::Initialized>(lib_hacky::on_initialized)
                .notification::<notification::DidChangeConfiguration>(
                    lib_hacky::on_did_change_configuration,
                )
                .notification::<notification::DidOpenTextDocument>(
                    lib_hacky::on_did_open_text_document,
                )
                .notification::<notification::DidChangeTextDocument>(
                    lib_hacky::on_did_change_text_document,
                )
                .notification::<notification::DidCloseTextDocument>(
                    lib_hacky::on_did_close_text_document,
                )
                .notification::<notification::DidSaveTextDocument>(
                    lib_hacky::on_did_save_text_document,
                )
                .notification::<notification::Exit>(lib_hacky::on_exit);
            return Self { router };
        }

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

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
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

fn on_shutdown(
    _state: &mut LspState,
    _params: (),
) -> impl Future<Output = Result<(), ResponseError>> {
    async { Ok(()) }
}

fn on_code_lens_request(
    state: &mut LspState,
    params: CodeLensParams,
) -> impl Future<Output = Result<Option<Vec<CodeLens>>, ResponseError>> {
    let file_path = &params.text_document.uri.to_file_path().unwrap();

    let mut context = match &state.root_path {
        Some(root_path) => {
            let fm = FileManager::new(root_path);
            let graph = CrateGraph::default();
            Context::new(fm, graph)
        }
        None => {
            let err = ResponseError::new(
                ErrorCode::REQUEST_FAILED,
                "Unable to determine the project root path",
            );
            return future::ready(Err(err));
        }
    };

    let crate_id = create_local_crate(&mut context, file_path, CrateType::Binary);

    // We ignore the warnings and errors produced by compilation for producing codelenses
    // because we can still get the test functions even if compilation fails
    let _ = check_crate(&mut context, crate_id, false, false);

    let fm = &context.file_manager;
    let files = fm.as_simple_files();
    let tests = context.get_all_test_functions_in_crate_matching(&crate_id, "");

    let mut lenses: Vec<CodeLens> = vec![];
    for func_id in tests {
        let location = context.function_meta(&func_id).name.location;
        let file_id = location.file;
        // TODO(#1681): This file_id never be 0 because the "path" where it maps is the directory, not a file
        if file_id.as_usize() != 0 {
            continue;
        }

        let func_name = context.function_name(&func_id);

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

fn on_did_save_text_document(
    state: &mut LspState,
    params: DidSaveTextDocumentParams,
) -> ControlFlow<Result<(), async_lsp::Error>> {
    let file_path = &params.text_document.uri.to_file_path().unwrap();
    let mut context = match &state.root_path {
        Some(root_path) => {
            let fm = FileManager::new(root_path);
            let graph = CrateGraph::default();
            Context::new(fm, graph)
        }
        None => {
            let err = ResponseError::new(
                ErrorCode::REQUEST_FAILED,
                "Unable to determine the project root path",
            );
            return ControlFlow::Break(Err(err.into()));
        }
    };

    let crate_id = create_local_crate(&mut context, file_path, CrateType::Binary);

    let mut diagnostics = Vec::new();

    let file_diagnostics = match check_crate(&mut context, crate_id, false, false) {
        Ok(warnings) => warnings,
        Err(errors_and_warnings) => errors_and_warnings,
    };

    if !file_diagnostics.is_empty() {
        let fm = &context.file_manager;
        let files = fm.as_simple_files();

        for FileDiagnostic { file_id, diagnostic } in file_diagnostics {
            // TODO(#1681): This file_id never be 0 because the "path" where it maps is the directory, not a file
            if file_id.as_usize() != 0 {
                continue;
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
