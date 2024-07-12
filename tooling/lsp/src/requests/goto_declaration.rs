use std::future::{self, Future};

use crate::types::GotoDeclarationResult;
use crate::LspState;
use async_lsp::ResponseError;

use lsp_types::request::{GotoDeclarationParams, GotoDeclarationResponse};

use super::{process_request, to_lsp_location};

pub(crate) fn on_goto_declaration_request(
    state: &mut LspState,
    params: GotoDeclarationParams,
) -> impl Future<Output = Result<GotoDeclarationResult, ResponseError>> {
    let result = on_goto_definition_inner(state, params);
    future::ready(result)
}

fn on_goto_definition_inner(
    state: &mut LspState,
    params: GotoDeclarationParams,
) -> Result<GotoDeclarationResult, ResponseError> {
    process_request(state, params.text_document_position_params, |location, interner, files, _| {
        interner.get_declaration_location_from(location).and_then(|found_location| {
            let file_id = found_location.file;
            let definition_position = to_lsp_location(files, file_id, found_location.span)?;
            let response = GotoDeclarationResponse::from(definition_position).to_owned();
            Some(response)
        })
    })
}
