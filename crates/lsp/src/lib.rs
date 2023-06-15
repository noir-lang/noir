use std::{
    future::Future,
    ops::ControlFlow,
    pin::Pin,
    task::{Context, Poll},
};

use acvm::Language;
use async_lsp::{
    router::Router, AnyEvent, AnyNotification, AnyRequest, ClientSocket, Error, LanguageClient,
    LspService, ResponseError,
};
use lsp_types::{
    notification, request, Diagnostic, DiagnosticSeverity, DidChangeConfigurationParams,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, InitializeParams, InitializeResult, InitializedParams,
    PublishDiagnosticsParams, Range, ServerCapabilities, TextDocumentSyncOptions,
};
use noirc_driver::Driver;
use noirc_errors::{DiagnosticKind, FileDiagnostic};
use noirc_frontend::graph::CrateType;
use serde_json::Value as JsonValue;
use tower::Service;

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
        let text_document_sync = TextDocumentSyncOptions {
            save: Some(true.into()),
            ..TextDocumentSyncOptions::default()
        };

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(text_document_sync.into()),
                // Add capabilities before this spread when adding support for one
                ..ServerCapabilities::default()
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
    // TODO: Requiring `Language` and `is_opcode_supported` to construct a driver makes for some real stinky code
    // The driver should not require knowledge of the backend; instead should be implemented as an independent pass (in nargo?)
    let mut driver = Driver::new(&Language::R1CS, Box::new(|_op| false));

    let file_path = &params.text_document.uri.to_file_path().unwrap();

    driver.create_local_crate(file_path, CrateType::Binary);

    let mut diagnostics = Vec::new();

    let file_diagnostics = match driver.check_crate(false) {
        Ok(warnings) => warnings,
        Err(errors_and_warnings) => errors_and_warnings,
    };

    if !file_diagnostics.is_empty() {
        let fm = driver.file_manager();
        let files = fm.as_simple_files();

        for FileDiagnostic { file_id, diagnostic } in file_diagnostics {
            // TODO: This file_id never be 0 because the "path" where it maps is the directory, not a file
            if file_id.as_usize() != 0 {
                continue;
            }

            let mut range = Range::default();

            // TODO: Should this be the first item in secondaries? Should we bail when we find a range?
            for sec in diagnostic.secondaries {
                // TODO: Codespan ranges are often (always?) off by some amount of characters
                if let Ok(codespan_range) =
                    codespan_lsp::byte_span_to_range(files, file_id.as_usize(), sec.span.into())
                {
                    // We have to manually attach each because the codespan_lsp restricts lsp-types to the wrong version range
                    range.start.line = codespan_range.start.line;
                    range.start.character = codespan_range.start.character;
                    range.end.line = codespan_range.end.line;
                    range.end.character = codespan_range.end.character;
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
                ..
            }
        ));
        assert!(response.server_info.is_none());
    }
}
