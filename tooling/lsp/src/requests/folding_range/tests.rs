use super::*;
use async_lsp::lsp_types::{
    FoldingRangeKind, PartialResultParams, TextDocumentIdentifier, Url, WorkDoneProgressParams,
};

fn get_folding_ranges(src: &str) -> Vec<FoldingRange> {
    let uri = Url::parse("file:///main.nr").unwrap();
    let input_files = HashMap::from([(uri.to_string(), src.to_string())]);

    on_folding_range_request(
        &input_files,
        FoldingRangeParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
            partial_result_params: PartialResultParams { partial_result_token: None },
        },
    )
    .expect("Could not execute on_folding_range_request")
    .unwrap()
}

#[test]
fn test_block_comment() {
    let src = "
        fn foo() {}

        /* This is a
           block 
           comment */

        fn bar() {}
        ";
    let ranges = get_folding_ranges(src);
    assert_eq!(ranges.len(), 1);

    let range = &ranges[0];
    assert_eq!(range.start_line, 3);
    assert_eq!(range.end_line, 5);
    assert_eq!(range.kind, Some(FoldingRangeKind::Comment));
}

#[test]
fn test_line_comment() {
    let src = "
        fn foo() {}

        // This is a
        // series of
        // consecutive comments

        // And this 
        // is another one

        fn bar() {}
        ";
    let ranges = get_folding_ranges(src);
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
fn test_does_not_mix_different_styles() {
    let src = "
        //! This should not
        //! be mixed with the next comment
        // This is a
        // series of
        // consecutive comments
        ";
    let ranges = get_folding_ranges(src);
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
fn test_series_of_mod() {
    let src = "
        mod one;
        mod two;

        mod three;
        mod four;
        mod five;
        ";
    let ranges = get_folding_ranges(src);
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
fn test_series_of_use() {
    let src = "
        use one;
        use two;

        use three;
        use four;
        use five;
        ";
    let ranges = get_folding_ranges(src);
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
fn test_use_list() {
    let src = "
        use one::{
            two::{
                three,
                four
            },
        };
        ";
    let ranges = get_folding_ranges(src);

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
fn test_series_of_use_when_there_is_a_list() {
    let src = "
        use one;
        use two::{
          three,
        };
        ";
    let ranges = get_folding_ranges(src);
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
