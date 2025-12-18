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
    assert_eq!(range.kind, None);

    let range = &ranges[1];
    assert_eq!(range.start_line, 4);
    assert_eq!(range.end_line, 6);
    assert_eq!(range.kind, None);
}

#[test]
async fn test_series_of_use() {
    let src = "
        use one;
        use two;

        use three;
        use four;
        use five;
        ";
    let ranges = get_folding_ranges(src).await;
    assert_eq!(ranges.len(), 2);

    let range = &ranges[0];
    assert_eq!(range.start_line, 1);
    assert_eq!(range.end_line, 2);
    assert_eq!(range.kind, Some(FoldingRangeKind::Imports));

    let range = &ranges[1];
    assert_eq!(range.start_line, 4);
    assert_eq!(range.end_line, 6);
    assert_eq!(range.kind, Some(FoldingRangeKind::Imports));
}

#[test]
async fn test_use_vector() {
    let src = "
        use one::{
            two::{
                three,
                four
            },
        };
        ";
    let ranges = get_folding_ranges(src).await;

    assert_eq!(ranges.len(), 2);

    let range = &ranges[0];
    assert_eq!(range.start_line, 1);
    assert_eq!(range.end_line, 6);
    assert_eq!(range.kind, Some(FoldingRangeKind::Imports));

    let range = &ranges[1];
    assert_eq!(range.start_line, 2);
    assert_eq!(range.end_line, 5);
    assert_eq!(range.kind, Some(FoldingRangeKind::Imports));
}

#[test]
async fn test_series_of_use_when_there_is_a_vector() {
    let src = "
        use one;
        use two::{
          three,
        };
        ";
    let ranges = get_folding_ranges(src).await;
    assert_eq!(ranges.len(), 2);

    let range = &ranges[0];
    assert_eq!(range.start_line, 2);
    assert_eq!(range.end_line, 4);
    assert_eq!(range.kind, Some(FoldingRangeKind::Imports));

    let range = &ranges[1];
    assert_eq!(range.start_line, 1);
    assert_eq!(range.end_line, 4);
    assert_eq!(range.kind, Some(FoldingRangeKind::Imports));
}
