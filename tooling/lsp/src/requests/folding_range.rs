use std::future;

use async_lsp::{
    ResponseError,
    lsp_types::{FoldingRange, FoldingRangeParams, Position, TextDocumentPositionParams},
};

use crate::{
    LspState,
    requests::{folding_range::comments_collector::FoldingRangeCommentsCollector, process_request},
};

mod comments_collector;

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

        let comments_collector = FoldingRangeCommentsCollector::new(file_id, args.files);
        let comment_ranges = comments_collector.collect(source);

        Some(comment_ranges)
    });

    future::ready(result)
}

#[cfg(test)]
mod tests {
    use crate::{notifications::on_did_open_text_document, test_utils};

    use super::*;
    use async_lsp::lsp_types::{
        DidOpenTextDocumentParams, FoldingRangeKind, PartialResultParams, TextDocumentIdentifier,
        TextDocumentItem, WorkDoneProgressParams,
    };
    use tokio::test;

    async fn get_folding_ranges(src: &str) -> Vec<FoldingRange> {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("document_symbol").await;

        let _ = on_did_open_text_document(
            &mut state,
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: noir_text_document.clone(),
                    language_id: "noir".to_string(),
                    version: 0,
                    text: src.to_string(),
                },
            },
        );

        on_folding_range_request(
            &mut state,
            FoldingRangeParams {
                text_document: TextDocumentIdentifier { uri: noir_text_document },
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
                partial_result_params: PartialResultParams { partial_result_token: None },
            },
        )
        .await
        .expect("Could not execute on_folding_range_request")
        .unwrap()
    }

    #[test]
    async fn test_block_comment() {
        let src = "
        fn foo() {}

        /* This is a
           block 
           comment */

        fn bar() {}
        ";
        let ranges = get_folding_ranges(src).await;
        assert_eq!(ranges.len(), 1);

        let range = &ranges[0];
        assert_eq!(range.start_line, 3);
        assert_eq!(range.end_line, 5);
        assert_eq!(range.kind, Some(FoldingRangeKind::Comment));
    }

    #[test]
    async fn test_line_comment() {
        let src = "
        fn foo() {}

        // This is a
        // series of
        // consecutive comments

        // And this 
        // is another one

        fn bar() {}
        ";
        let ranges = get_folding_ranges(src).await;
        assert_eq!(ranges.len(), 2);

        let range = &ranges[0];
        assert_eq!(range.start_line, 3);
        assert_eq!(range.end_line, 5);
        assert_eq!(range.kind, Some(FoldingRangeKind::Comment));

        let range = &ranges[1];
        assert_eq!(range.start_line, 7);
        assert_eq!(range.end_line, 8);
        assert_eq!(range.kind, Some(FoldingRangeKind::Comment));
    }

    #[test]
    async fn test_does_not_mix_different_styles() {
        let src = "
        //! This should not
        //! be mixed with the next comment
        // This is a
        // series of
        // consecutive comments
        ";
        let ranges = get_folding_ranges(src).await;
        assert_eq!(ranges.len(), 2);

        let range = &ranges[0];
        assert_eq!(range.start_line, 1);
        assert_eq!(range.end_line, 2);
        assert_eq!(range.kind, Some(FoldingRangeKind::Comment));

        let range = &ranges[1];
        assert_eq!(range.start_line, 3);
        assert_eq!(range.end_line, 5);
        assert_eq!(range.kind, Some(FoldingRangeKind::Comment));
    }

    #[test]
    async fn test_series_of_mod() {
        let src = "
        mod one;
        mod two;

        mod three;
        mod four;
        mod five;
        ";
        let ranges = get_folding_ranges(src).await;
        assert_eq!(ranges.len(), 2);

        let range = &ranges[0];
        assert_eq!(range.start_line, 1);
        assert_eq!(range.end_line, 2);
        assert_eq!(range.kind, Some(FoldingRangeKind::Comment));

        let range = &ranges[1];
        assert_eq!(range.start_line, 4);
        assert_eq!(range.end_line, 6);
        assert_eq!(range.kind, Some(FoldingRangeKind::Comment));
    }
}
