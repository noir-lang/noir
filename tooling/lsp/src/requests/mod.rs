use std::future::Future;

use crate::types::{CodeLensOptions, InitializeParams};
use async_lsp::ResponseError;
use lsp_types::{Position, TextDocumentSyncCapability, TextDocumentSyncKind};
use nargo_fmt::Config;
use serde::{Deserialize, Serialize};

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
mod goto_definition;
mod profile_run;
mod test_run;
mod tests;

pub(crate) use {
    code_lens_request::collect_lenses_for_package, code_lens_request::on_code_lens_request,
    goto_definition::on_goto_definition_request, profile_run::on_profile_run_request,
    test_run::on_test_run_request, tests::on_tests_request,
};

/// LSP client will send initialization request after the server has started.
/// [InitializeParams].`initialization_options` will contain the options sent from the client.
#[derive(Debug, Deserialize, Serialize)]
struct LspInitializationOptions {
    /// Controls whether code lens is enabled by the server
    /// By default this will be set to true (enabled).
    #[serde(rename = "enableCodeLens", default = "default_enable_code_lens")]
    enable_code_lens: bool,
}

fn default_enable_code_lens() -> bool {
    true
}

impl Default for LspInitializationOptions {
    fn default() -> Self {
        Self { enable_code_lens: default_enable_code_lens() }
    }
}

pub(crate) fn on_initialize(
    state: &mut LspState,
    params: InitializeParams,
) -> impl Future<Output = Result<InitializeResult, ResponseError>> {
    state.root_path = params.root_uri.and_then(|root_uri| root_uri.to_file_path().ok());

    let initialization_options: LspInitializationOptions = params
        .initialization_options
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();

    async move {
        let text_document_sync = TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL);

        let code_lens = if initialization_options.enable_code_lens {
            Some(CodeLensOptions { resolve_provider: Some(false) })
        } else {
            None
        };

        let nargo = NargoCapability {
            tests: Some(NargoTestsOptions {
                fetch: Some(true),
                run: Some(true),
                update: Some(true),
            }),
        };

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(text_document_sync),
                code_lens_provider: code_lens,
                document_formatting_provider: true,
                nargo: Some(nargo),
                definition_provider: Some(lsp_types::OneOf::Left(true)),
            },
            server_info: None,
        })
    }
}

pub(crate) fn on_formatting(
    state: &mut LspState,
    params: lsp_types::DocumentFormattingParams,
) -> impl Future<Output = Result<Option<Vec<lsp_types::TextEdit>>, ResponseError>> {
    std::future::ready(on_formatting_inner(state, params))
}

fn on_formatting_inner(
    state: &LspState,
    params: lsp_types::DocumentFormattingParams,
) -> Result<Option<Vec<lsp_types::TextEdit>>, ResponseError> {
    let path = params.text_document.uri.to_string();

    if let Some(source) = state.input_files.get(&path) {
        let (module, errors) = noirc_frontend::parse_program(source);
        if !errors.is_empty() {
            return Ok(None);
        }

        let new_text = nargo_fmt::format(source, module, &Config::default());

        let start_position = Position { line: 0, character: 0 };
        let end_position = Position {
            line: source.lines().count() as u32,
            character: source.chars().count() as u32,
        };

        Ok(Some(vec![lsp_types::TextEdit {
            range: lsp_types::Range::new(start_position, end_position),
            new_text,
        }]))
    } else {
        Ok(None)
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
        CodeLensOptions, InitializeParams, TextDocumentSyncCapability, TextDocumentSyncKind,
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
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL
                )),
                code_lens_provider: Some(CodeLensOptions { resolve_provider: Some(false) }),
                document_formatting_provider: true,
                ..
            }
        ));
        assert!(response.server_info.is_none());
    }
}
