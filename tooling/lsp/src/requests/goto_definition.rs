use std::future::{self, Future};

use crate::{types::GotoDefinitionResult, LspState};
use async_lsp::{ErrorCode, LanguageClient, ResponseError};
use fm::codespan_files::Error;
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location};
use lsp_types::{Position, Url};
use nargo_toml::{find_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::NOIR_ARTIFACT_VERSION_STRING;

pub(crate) fn on_goto_definition_request(
    state: &mut LspState,
    params: GotoDefinitionParams,
) -> impl Future<Output = Result<GotoDefinitionResult, ResponseError>> {
    let result = on_goto_definition_inner(state, params);
    future::ready(result)
}

fn on_goto_definition_inner(
    state: &mut LspState,
    params: GotoDefinitionParams,
) -> Result<GotoDefinitionResult, ResponseError> {
    let root_path = state.root_path.as_deref().ok_or_else(|| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "Could not find project root")
    })?;

    let file_path =
        params.text_document_position_params.text_document.uri.to_file_path().map_err(|_| {
            ResponseError::new(ErrorCode::REQUEST_FAILED, "URI is not a valid file path")
        })?;

    let toml_path = match find_package_manifest(root_path, &file_path) {
        Ok(toml_path) => toml_path,
        Err(err) => {
            let _ = state.client.log_message(lsp_types::LogMessageParams {
                typ: lsp_types::MessageType::WARNING,
                message: err.to_string(),
            });
            return Ok(None);
        }
    };
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        PackageSelection::All,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )
    .map_err(|err| {
        // If we found a manifest, but the workspace is invalid, we raise an error about it
        ResponseError::new(ErrorCode::REQUEST_FAILED, err)
    })?;

    let mut definition_position = None;

    for package in &workspace {
        let (mut context, crate_id) = nargo::prepare_package(package);

        // We ignore the warnings and errors produced by compilation while resolving the definition
        let _ = noirc_driver::check_crate(&mut context, crate_id, false, false);

        let files = context.file_manager.as_file_map();
        let file_id = context.file_manager.name_to_id(file_path.clone());

        if let Some(file_id) = file_id {
            let byte_index = position_to_byte_index(
                files,
                file_id,
                &params.text_document_position_params.position,
            );

            if let Ok(byte_index) = byte_index {
                let search_for_location = noirc_errors::Location {
                    file: file_id,
                    span: noirc_errors::Span::single_char(byte_index as u32),
                };
                let found_location = context.get_definition_location_from(search_for_location);

                if let Some(found_location) = found_location {
                    let file_id = found_location.file;
                    definition_position = to_lsp_location(files, file_id, found_location.span);
                }
            }
        }
    }

    if let Some(definition_position) = definition_position {
        let response: GotoDefinitionResponse =
            GotoDefinitionResponse::from(definition_position).to_owned();
        Ok(Some(response))
    } else {
        Ok(None)
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

#[cfg(test)]
mod goto_definition_tests {

    use async_lsp::ClientSocket;
    use tokio::test;

    use crate::solver::MockBackend;

    use super::*;

    #[test]
    async fn test_on_goto_definition() {
        let client = ClientSocket::new_closed();
        let solver = MockBackend;
        let mut state = LspState::new(&client, solver);

        let root_path = std::env::current_dir()
            .unwrap()
            .join("../../test_programs/execution_success/7_function")
            .canonicalize()
            .expect("Could not resolve root path");
        let noir_text_document = Url::from_file_path(root_path.join("src/main.nr").as_path())
            .expect("Could not convert text document path to URI");
        let root_uri = Some(
            Url::from_file_path(root_path.as_path()).expect("Could not convert root path to URI"),
        );

        #[allow(deprecated)]
        let initialize_params = lsp_types::InitializeParams {
            process_id: Default::default(),
            root_path: None,
            root_uri,
            initialization_options: None,
            capabilities: Default::default(),
            trace: Some(lsp_types::TraceValue::Verbose),
            workspace_folders: None,
            client_info: None,
            locale: None,
        };
        let _initialize_response = crate::requests::on_initialize(&mut state, initialize_params)
            .await
            .expect("Could not initialize LSP server");

        let params = GotoDefinitionParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: noir_text_document },
                position: Position { line: 95, character: 5 },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let response = on_goto_definition_request(&mut state, params)
            .await
            .expect("Could execute on_goto_definition_request");

        assert!(&response.is_some());
    }
}
