use std::{future::{self, Future}, path::{Path}};

use async_lsp::{ResponseError, ErrorCode};
use codespan_reporting::files::Error;
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location};
use nargo_toml::{find_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::NOIR_ARTIFACT_VERSION_STRING;

use crate::LspState;
use lsp_types::{Position, Url};

pub(crate) fn on_goto_definition_request(
    state: &mut LspState,
    params: GotoDefinitionParams,
) -> impl Future<Output = Result<Option<GotoDefinitionResponse>, ResponseError>> {
    future::ready(on_goto_definition_inner(state, params))
}

fn on_goto_definition_inner(
    state: &mut LspState,
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>, ResponseError> {

    let root_path = state.root_path.as_deref().ok_or_else(|| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "Could not find project root")
    })?;

    find_definition_location(root_path, params)    
}

fn find_definition_location(root_path: &Path, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>, ResponseError> {
    let file_path = params.text_document_position_params.text_document.uri.to_file_path().map_err(|_| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "URI is not a valid file path")
    })?;


    let toml_path = match find_package_manifest(root_path, &file_path) {
        Ok(toml_path) => toml_path,
        Err(_err) => {
            // If we cannot find a manifest, we log a warning but return no code lenses
            // We can reconsider this when we can build a file without the need for a Nargo.toml file to resolve deps
            // let _ = state.client.log_message(LogMessageParams {
            //     typ: MessageType::WARNING,
            //     message: err.to_string(),
            // });
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
        let (mut context, crate_id) = nargo::prepare_package(package, Box::new(crate::get_non_stdlib_asset));
        // We ignore the warnings and errors produced by compilation for producing code lenses
        // because we can still get the test functions even if compilation fails
        let _ = noirc_driver::check_crate(&mut context, crate_id, false);

        let files = context.file_manager.as_file_map();
        let file_id = context.file_manager.name_to_id(file_path.clone()).unwrap();

        let byte_index = position_to_byte_index(files, file_id, &params.text_document_position_params.position);

        println!("Looking up Byte Index {:?}", byte_index);
        if let Ok(byte_index) = byte_index {
        
            let found_location = context.find_definition_location(file_id, &noirc_errors::Span::single_char(byte_index as u32));
        
            if let Some(found_location) = found_location {
                let file_id = found_location.file;
                definition_position = to_lsp_location(files, file_id, found_location.span);
            } 
        }

    }

    if let Some(definition_position) = definition_position {
        let response: GotoDefinitionResponse = GotoDefinitionResponse::from(definition_position);
        Ok(Some(response))
    } else {
        Ok(None)
    }
}

fn to_lsp_location<'a, F>(files: &'a F, file_id: F::FileId, definition_span: noirc_errors::Span) -> Option<Location> 
where
    F: codespan_reporting::files::Files<'a> + ?Sized,
{
    println!("Definition Span {:?}", definition_span);
    let range = crate::byte_span_to_range(files, file_id,  definition_span.into());

    if let Some(range) = range {

        let file_name = files.name(file_id).unwrap();
        let path = file_name.to_string();
        let uri = Url::from_file_path(path).unwrap();

        Some(Location {
            uri,
            range,
        })
    } else {
        None
    }

}

pub(crate) fn position_to_byte_index<'a, F>(
    files: &'a F,
    file_id: F::FileId,
    position: &Position,
) -> Result<usize, Error>
where
    F: codespan_reporting::files::Files<'a> + ?Sized,
{
    let source = files.source(file_id)?;
    let source = source.as_ref();

    let line_span = files.line_range(file_id, position.line as usize).unwrap();
    println!("position to BI: line_span: {:?}", line_span);
    let line_str = source.get(line_span.clone()).unwrap();
    println!("position to BI: line_str: {:?}", line_str);

    let byte_offset = character_to_line_offset(line_str, position.character)?;

    Ok(line_span.start + byte_offset)
   
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
        Err(Error::ColumnTooLarge {
            given: character_offset as usize,
            max: line.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{path::{PathBuf, Path}, str::FromStr};

    use crate::solver::WrapperSolver;

    use super::*;

    #[test]
    fn test_goto_definition() {

        let root_path = PathBuf::from_str("/Users/koby/git/noir/tooling/nargo_cli/tests/execution_success/slice_struct_field").unwrap().clone();
        
        let params = GotoDefinitionParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier {
                    uri: Url::from_file_path(root_path.join("src/main.nr").as_path()).unwrap(),
                },
                position: Position {
                    line: 18,
                    character: 19,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let resp = find_definition_location(root_path.as_path(), params);

        debug_assert!(resp.is_ok());

        println!("{:?}", resp);

    }
}


