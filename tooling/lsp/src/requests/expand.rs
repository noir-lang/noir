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
    let is_stdlib = params.text_document.uri.scheme() == "noir-std";

    let text_document_position_params = TextDocumentPositionParams {
        text_document: params.text_document,
        position: params.position,
    };

    let result = process_request(state, text_document_position_params, |args| {
        let crate_id = if is_stdlib { *args.crate_graph.stdlib_crate_id() } else { args.crate_id };
        get_expanded_crate(crate_id, args.crate_graph, args.def_maps, args.interner)
    });

    future::ready(result)
}
