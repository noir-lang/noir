use std::future::Future;

use crate::types::{CodeLensOptions, InitializeParams, TextDocumentSyncOptions};
use async_lsp::ResponseError;

use crate::{
    types::{InitializeResult, NargoCapability, NargoTestsOptions, ServerCapabilities},
    LspState,
};

// Handlers
// The handlers for `request` are not `async` because it compiles down to lifetimes that can't be added to
// the router. To return a future that fits the trait, it is easiest wrap your implementations in an `async {}`
// block but you can also use `std::future::ready`.
//
// Additionally, the handlers for `notification` aren't async at all.
//
// They are not attached to the `NargoLspService` struct so they can be unit tested with only `LspState`
// and params passed in.

mod code_lens_request;
mod test_run;
mod tests;

pub(crate) use {
    code_lens_request::on_code_lens_request, test_run::on_test_run_request, tests::on_tests_request,
};

pub(crate) fn on_initialize(
    state: &mut LspState,
    params: InitializeParams,
) -> impl Future<Output = Result<InitializeResult, ResponseError>> {
    state.root_path = params.root_uri.and_then(|root_uri| root_uri.to_file_path().ok());

    async {
        let text_document_sync =
            TextDocumentSyncOptions { save: Some(true.into()), ..Default::default() };

        let code_lens = CodeLensOptions { resolve_provider: Some(false) };

        let nargo = NargoCapability {
            tests: Some(NargoTestsOptions {
                fetch: Some(true),
                run: Some(true),
                update: Some(true),
            }),
        };

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(text_document_sync.into()),
                code_lens_provider: Some(code_lens),
                nargo: Some(nargo),
            },
            server_info: None,
        })
    }
}

pub(crate) fn on_shutdown(
    _state: &mut LspState,
    _params: (),
) -> impl Future<Output = Result<(), ResponseError>> {
    async { Ok(()) }
}

#[cfg(test)]
mod initialization {
    use async_lsp::ClientSocket;
    use lsp_types::{
        CodeLensOptions, InitializeParams, TextDocumentSyncCapability, TextDocumentSyncOptions,
    };
    use tokio::test;

    use crate::{
        requests::on_initialize, solver::MockBackend, types::ServerCapabilities, LspState,
    };

    #[test]
    async fn test_on_initialize() {
        let client = ClientSocket::new_closed();
        let solver = MockBackend;
        let mut state = LspState::new(&client, solver);
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
