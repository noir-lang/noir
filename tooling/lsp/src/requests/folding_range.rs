use std::future;

use async_lsp::{
    ResponseError,
    lsp_types::{FoldingRange, FoldingRangeParams, Position, TextDocumentPositionParams},
};

use crate::{LspState, requests::process_request};

pub(crate) fn on_folding_range_request(
    state: &mut LspState,
    params: FoldingRangeParams,
) -> impl Future<Output = Result<Option<Vec<FoldingRange>>, ResponseError>> + use<> {
    let text_document_position_params = TextDocumentPositionParams {
        text_document: params.text_document.clone(),
        position: Position { line: 0, character: 0 },
    };

    let result = process_request(state, text_document_position_params, |args| {
        let file_id = args.location.file;
        let file = args.files.get_file(file_id).unwrap();
        let source = file.source();
        None
    });

    future::ready(result)
}
