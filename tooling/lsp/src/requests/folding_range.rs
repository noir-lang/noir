use async_lsp::{
    ErrorCode, ResponseError,
    lsp_types::{FoldingRange, FoldingRangeParams, Position, TextDocumentPositionParams},
};

use crate::{
    LspState, PendingRequest,
    requests::{
        folding_range::{comments_collector::CommentsCollector, nodes_collector::NodesCollector},
        process_request,
    },
};

mod comments_collector;
mod nodes_collector;
#[cfg(test)]
mod tests;

pub(crate) fn on_folding_range_request(
    state: &mut LspState,
    params: FoldingRangeParams,
) -> impl Future<Output = Result<Option<Vec<FoldingRange>>, ResponseError>> + use<> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    if state.pending_type_check_events == 0 {
        let _ = tx.send(on_folding_range_request_inner(state, params));
    } else {
        state.pending_requests.push(PendingRequest::FoldingRange { params, tx });
    }

    async move {
        rx.await.map_err(|_| {
            let msg = "Folding ragne request failed".to_string();
            ResponseError::new(ErrorCode::REQUEST_FAILED, msg)
        })?
    }
}

pub(crate) fn on_folding_range_request_inner(
    state: &mut LspState,
    params: FoldingRangeParams,
) -> Result<Option<Vec<FoldingRange>>, ResponseError> {
    let text_document_position_params = TextDocumentPositionParams {
        text_document: params.text_document.clone(),
        position: Position { line: 0, character: 0 },
    };

    process_request("folding_range", state, text_document_position_params, |args| {
        let file_id = args.location.file;
        let file = args.files.get_file(file_id).unwrap();
        let source = file.source();

        let comments_collector = CommentsCollector::new(file_id, args.files);
        let mut ranges = comments_collector.collect(source);

        let nodes_collector = NodesCollector::new(file_id, args.files);
        let node_ranges = nodes_collector.collect(source);

        ranges.extend(node_ranges);

        Some(ranges)
    })
}
