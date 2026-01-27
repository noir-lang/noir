use std::future::Future;

use crate::types::GotoDeclarationResult;
use crate::{LspState, PendingRequest, PendingRequestKind};
use async_lsp::{ErrorCode, ResponseError};

use async_lsp::lsp_types::request::{GotoDeclarationParams, GotoDeclarationResponse};

use super::{process_request, to_lsp_location};

pub(crate) fn on_goto_declaration_request(
    state: &mut LspState,
    params: GotoDeclarationParams,
) -> impl Future<Output = Result<GotoDeclarationResult, ResponseError>> + use<> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    if state.pending_type_check_events == 0 {
        let _ = tx.send(on_goto_declaration_request_inner(state, params));
    } else {
        let type_check_version = state.type_check_version;
        state.pending_requests.push(PendingRequest::new(
            PendingRequestKind::GotoDeclaration { params, tx },
            type_check_version,
        ));
    }

    async move {
        rx.await.map_err(|_| {
            let msg = "Goto declaration request failed".to_string();
            ResponseError::new(ErrorCode::REQUEST_FAILED, msg)
        })?
    }
}

pub(crate) fn on_goto_declaration_request_inner(
    state: &mut LspState,
    params: GotoDeclarationParams,
) -> Result<GotoDeclarationResult, ResponseError> {
    process_request("goto_definition", state, params.text_document_position_params, |args| {
        args.interner.get_declaration_location_from(args.location).and_then(|found_location| {
            let file_id = found_location.file;
            let definition_position = to_lsp_location(args.files, file_id, found_location.span)?;
            let response = GotoDeclarationResponse::from(definition_position).to_owned();
            Some(response)
        })
    })
}
