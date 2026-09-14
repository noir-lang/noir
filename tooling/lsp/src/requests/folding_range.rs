use std::collections::HashMap;
use std::path::PathBuf;

use async_lsp::{
    ResponseError,
    lsp_types::{FoldingRange, FoldingRangeParams},
};
use fm::{FileMap, PathString};

use crate::requests::folding_range::{
    comments_collector::CommentsCollector, nodes_collector::NodesCollector,
};

mod comments_collector;
mod nodes_collector;
#[cfg(test)]
mod tests;

/// Like formatting, this request is parse-only: it takes the open documents' current texts
/// instead of `LspState`, so the main loop answers it directly from its text mirror instead
/// of queueing it behind type-checking.
pub(crate) fn on_folding_range_request(
    input_files: &HashMap<String, String>,
    params: FoldingRangeParams,
) -> Result<Option<Vec<FoldingRange>>, ResponseError> {
    let uri = params.text_document.uri;
    let Some(source) = input_files.get(&uri.to_string()) else {
        return Ok(None);
    };

    let mut files = FileMap::default();
    let file_id = files.add_file(PathString::from_path(PathBuf::from(uri.path())), source.clone());

    let comments_collector = CommentsCollector::new(file_id, &files);
    let mut ranges = comments_collector.collect(source);

    let nodes_collector = NodesCollector::new(file_id, &files);
    let node_ranges = nodes_collector.collect(source);

    ranges.extend(node_ranges);

    Ok(Some(ranges))
}
