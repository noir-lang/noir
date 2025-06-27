use std::future;

use async_lsp::{ResponseError, lsp_types::TextDocumentPositionParams};
use nargo_expand::get_expanded_crate;

use crate::{
    LspState,
    requests::process_request,
    types::{NargoExpandParams, NargoExpandResult},
};

pub(crate) fn on_expand_request(
    state: &mut LspState,
    params: NargoExpandParams,
) -> impl Future<Output = Result<NargoExpandResult, ResponseError>> + use<> {
    let text_document_position_params = TextDocumentPositionParams {
        text_document: params.text_document,
        position: params.position,
    };

    let result = process_request(state, text_document_position_params, |args| {
        get_expanded_crate(args.crate_id, args.crate_graph, args.def_maps, args.interner)
    });

    future::ready(result)
}
