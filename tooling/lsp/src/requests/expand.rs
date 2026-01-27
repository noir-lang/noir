use async_lsp::{ErrorCode, ResponseError, lsp_types::TextDocumentPositionParams};
use nargo_expand::get_expanded_crate;

use crate::{
    LspState, PendingRequest,
    requests::process_request,
    types::{NargoExpandParams, NargoExpandResult},
};

pub(crate) fn on_expand_request(
    state: &mut LspState,
    params: NargoExpandParams,
) -> impl Future<Output = Result<NargoExpandResult, ResponseError>> + use<> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    if state.pending_type_check_events == 0 {
        let _ = tx.send(on_expand_request_inner(state, params));
    } else {
        state.pending_requests.push(PendingRequest::Expand { params, tx });
    }

    async move {
        rx.await.map_err(|_| {
            let msg = "Expand request failed".to_string();
            ResponseError::new(ErrorCode::REQUEST_FAILED, msg)
        })?
    }
}

pub(crate) fn on_expand_request_inner(
    state: &mut LspState,
    params: NargoExpandParams,
) -> Result<NargoExpandResult, ResponseError> {
    let is_stdlib = params.text_document.uri.scheme() == "noir-std";

    let text_document_position_params = TextDocumentPositionParams {
        text_document: params.text_document,
        position: params.position,
    };

    process_request("expand", state, text_document_position_params, |args| {
        let crate_id = if is_stdlib { *args.crate_graph.stdlib_crate_id() } else { args.crate_id };
        get_expanded_crate(crate_id, args.crate_graph, args.def_maps, args.interner)
    })
}
