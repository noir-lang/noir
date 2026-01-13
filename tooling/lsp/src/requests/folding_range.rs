use std::future;

use async_lsp::{
    ResponseError,
    lsp_types::{FoldingRange, FoldingRangeParams, Position, TextDocumentPositionParams},
};

use crate::{
    LspState,
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
    let text_document_position_params = TextDocumentPositionParams {
        text_document: params.text_document.clone(),
        position: Position { line: 0, character: 0 },
    };

    let result = process_request(state, text_document_position_params, |args| {
        let file_id = args.location.file;
        let file = args.files.get_file(file_id).unwrap();
        let source = file.source();

        let comments_collector = CommentsCollector::new(file_id, args.files);
        let mut ranges = comments_collector.collect(source);

        let nodes_collector = NodesCollector::new(file_id, args.files);
        let node_ranges = nodes_collector.collect(source);

        ranges.extend(node_ranges);

        Some(ranges)
    });

    future::ready(result)
}
