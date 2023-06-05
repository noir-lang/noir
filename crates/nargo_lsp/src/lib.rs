use std::{
    future::Future,
    ops::ControlFlow,
    pin::Pin,
    task::{Context, Poll},
};

use async_lsp::{
    router::Router, AnyEvent, AnyNotification, AnyRequest, Error, LspService, ResponseError,
};
use lsp_types::{
    notification, request, DidChangeConfigurationParams, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult,
    InitializedParams, ServerCapabilities,
};
use serde_json::Value as JsonValue;
use tower::Service;

// State for the LSP gets implemented on this struct and is internal to the implementation
#[derive(Debug, Default)]
struct LspState;

pub struct NargoLspService {
    router: Router<LspState>,
}

impl NargoLspService {
    pub fn new() -> Self {
        let state = LspState::default();
        let mut router = Router::new(state);
        router
            .request::<request::Initialize, _>(on_initialize)
            .notification::<notification::Initialized>(on_initialized)
            .notification::<notification::DidChangeConfiguration>(on_did_change_configuration)
            .notification::<notification::DidOpenTextDocument>(on_did_open_text_document)
            .notification::<notification::DidChangeTextDocument>(on_did_change_text_document)
            .notification::<notification::DidCloseTextDocument>(on_did_close_text_document);
        Self { router }
    }
}

impl Default for NargoLspService {
    fn default() -> Self {
        Self::new()
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
    fn notify(&mut self, notif: AnyNotification) -> ControlFlow<Result<(), Error>> {
        self.router.notify(notif)
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
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // Add capabilities before this spread when adding support for one
                ..ServerCapabilities::default()
            },
            server_info: None,
        })
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

#[cfg(test)]
mod lsp_tests {
    use tokio::test;

    use super::*;

    #[test]
    async fn test_on_initialize() {
        let mut state = LspState::default();
        let params = InitializeParams::default();
        let response = on_initialize(&mut state, params).await.unwrap();
        assert_eq!(response.capabilities, ServerCapabilities::default());
        assert!(response.server_info.is_none());
    }
}
