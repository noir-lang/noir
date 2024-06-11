use std::future::Future;

use crate::types::{CodeLensOptions, InitializeParams};
use async_lsp::{ErrorCode, ResponseError};
use fm::{codespan_files::Error, FileMap, PathString};
use lsp_types::{
    DeclarationCapability, Location, Position, TextDocumentSyncCapability, TextDocumentSyncKind,
    TypeDefinitionProviderCapability, Url,
};
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
mod goto_declaration;
mod goto_definition;
mod profile_run;
mod test_run;
mod tests;

pub(crate) use {
    code_lens_request::collect_lenses_for_package, code_lens_request::on_code_lens_request,
    goto_declaration::on_goto_declaration_request, goto_definition::on_goto_definition_request,
    goto_definition::on_goto_type_definition_request, profile_run::on_profile_run_request,
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

    #[serde(rename = "enableParsingCache", default = "default_enable_parsing_cache")]
    enable_parsing_cache: bool,
}

fn default_enable_code_lens() -> bool {
    true
}

fn default_enable_parsing_cache() -> bool {
    true
}

impl Default for LspInitializationOptions {
    fn default() -> Self {
        Self {
            enable_code_lens: default_enable_code_lens(),
            enable_parsing_cache: default_enable_parsing_cache(),
        }
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
    state.parsing_cache_enabled = initialization_options.enable_parsing_cache;

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
                declaration_provider: Some(DeclarationCapability::Simple(true)),
                type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
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

pub(crate) fn position_to_byte_index<'a, F>(
    files: &'a F,
    file_id: F::FileId,
    position: &Position,
) -> Result<usize, Error>
where
    F: fm::codespan_files::Files<'a> + ?Sized,
{
    let source = files.source(file_id)?;
    let source = source.as_ref();

    let line_span = files.line_range(file_id, position.line as usize)?;

    let line_str = source.get(line_span.clone());

    if let Some(line_str) = line_str {
        let byte_offset = character_to_line_offset(line_str, position.character)?;
        Ok(line_span.start + byte_offset)
    } else {
        Err(Error::InvalidCharBoundary { given: position.line as usize })
    }
}

fn position_to_location(
    files: &FileMap,
    file_path: &PathString,
    position: &Position,
) -> Result<noirc_errors::Location, ResponseError> {
    let file_id = files.get_file_id(file_path).ok_or(ResponseError::new(
        ErrorCode::REQUEST_FAILED,
        format!("Could not find file in file manager. File path: {:?}", file_path),
    ))?;
    let byte_index = position_to_byte_index(files, file_id, position).map_err(|err| {
        ResponseError::new(
            ErrorCode::REQUEST_FAILED,
            format!("Could not convert position to byte index. Error: {:?}", err),
        )
    })?;

    let location = noirc_errors::Location {
        file: file_id,
        span: noirc_errors::Span::single_char(byte_index as u32),
    };

    Ok(location)
}

fn character_to_line_offset(line: &str, character: u32) -> Result<usize, Error> {
    let line_len = line.len();
    let mut character_offset = 0;

    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        if character_offset == character {
            let chars_off = chars.as_str().len();
            let ch_off = ch.len_utf8();

            return Ok(line_len - chars_off - ch_off);
        }

        character_offset += ch.len_utf16() as u32;
    }

    // Handle positions after the last character on the line
    if character_offset == character {
        Ok(line_len)
    } else {
        Err(Error::ColumnTooLarge { given: character_offset as usize, max: line.len() })
    }
}

fn to_lsp_location<'a, F>(
    files: &'a F,
    file_id: F::FileId,
    definition_span: noirc_errors::Span,
) -> Option<Location>
where
    F: fm::codespan_files::Files<'a> + ?Sized,
{
    let range = crate::byte_span_to_range(files, file_id, definition_span.into())?;
    let file_name = files.name(file_id).ok()?;

    let path = file_name.to_string();
    let uri = Url::from_file_path(path).ok()?;

    Some(Location { uri, range })
}

pub(crate) fn on_shutdown(
    _state: &mut LspState,
    _params: (),
) -> impl Future<Output = Result<(), ResponseError>> {
    async { Ok(()) }
}

#[cfg(test)]
mod initialization {
    use acvm::blackbox_solver::StubbedBlackBoxSolver;
    use async_lsp::ClientSocket;
    use lsp_types::{
        CodeLensOptions, InitializeParams, TextDocumentSyncCapability, TextDocumentSyncKind,
    };
    use tokio::test;

    use crate::{requests::on_initialize, types::ServerCapabilities, LspState};

    #[test]
    async fn test_on_initialize() {
        let client = ClientSocket::new_closed();
        let mut state = LspState::new(&client, StubbedBlackBoxSolver);
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

#[cfg(test)]
mod character_to_line_offset_tests {
    use super::*;

    #[test]
    fn test_character_to_line_offset() {
        let line = "Hello, dark!";
        let character = 8;

        let result = character_to_line_offset(line, character).unwrap();
        assert_eq!(result, 8);

        // In the case of a multi-byte character, the offset should be the byte index of the character
        // byte offset for 8 character (黑) is expected to be 10
        let line = "Hello, 黑!";
        let character = 8;

        let result = character_to_line_offset(line, character).unwrap();
        assert_eq!(result, 10);
    }
}
