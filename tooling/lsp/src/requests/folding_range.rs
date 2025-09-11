use std::future;

use async_lsp::{
    ResponseError,
    lsp_types::{
        self, FoldingRange, FoldingRangeKind, FoldingRangeParams, Position,
        TextDocumentPositionParams,
    },
};
use fm::{FileId, FileMap};
use noirc_frontend::{
    lexer::Lexer,
    token::{DocStyle, Token},
};

use crate::{
    LspState,
    requests::{process_request, to_lsp_location},
};

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

struct FoldingRangeCommentsCollector<'files> {
    file_id: FileId,
    files: &'files FileMap,
    ranges: Vec<FoldingRange>,
    current_line_comment_group: Option<LineCommentGroup>,
}

struct LineCommentGroup {
    start: lsp_types::Location,
    end: lsp_types::Location,
    doc_style: Option<DocStyle>,
}

impl<'files> FoldingRangeCommentsCollector<'files> {
    fn new(file_id: FileId, files: &'files FileMap) -> Self {
        Self { file_id, files, ranges: Vec::new(), current_line_comment_group: None }
    }

    fn collect(mut self, source: &str) -> Vec<FoldingRange> {
        let lexer = Lexer::new(source, self.file_id).skip_comments(false);

        for token in lexer {
            let Ok(token) = token else {
                continue;
            };

            let location = token.location();

            match token.into_token() {
                Token::BlockComment(..) => {
                    self.push_current_line_comment_group();

                    let Some(location) = to_lsp_location(self.files, self.file_id, location.span)
                    else {
                        continue;
                    };

                    // Block comments are never grouped with other comments
                    self.ranges.push(FoldingRange {
                        start_line: location.range.start.line,
                        start_character: None,
                        end_line: location.range.end.line,
                        end_character: None,
                        kind: Some(FoldingRangeKind::Comment),
                        collapsed_text: None,
                    });
                }
                Token::LineComment(_, doc_style) => {
                    let Some(location) = to_lsp_location(self.files, self.file_id, location.span)
                    else {
                        continue;
                    };

                    if let Some(group) = &mut self.current_line_comment_group {
                        // Keep grouping while the line comment style is the same and they are consecutive lines
                        if group.doc_style == doc_style
                            && location.range.end.line - group.end.range.end.line <= 1
                        {
                            group.end = location;
                            continue;
                        }

                        // A new group starts, so push the current one
                        Self::push_line_comment_group(group, &mut self.ranges);
                    }

                    let start = location.clone();
                    self.current_line_comment_group =
                        Some(LineCommentGroup { start, end: location, doc_style });
                }
                _ => {
                    self.push_current_line_comment_group();
                }
            }
        }

        self.push_current_line_comment_group();

        self.ranges
    }

    fn push_current_line_comment_group(&mut self) {
        if let Some(group) = self.current_line_comment_group.take() {
            Self::push_line_comment_group(&group, &mut self.ranges);
        }
    }

    fn push_line_comment_group(group: &LineCommentGroup, ranges: &mut Vec<FoldingRange>) {
        ranges.push(FoldingRange {
            start_line: group.start.range.start.line,
            start_character: None,
            end_line: group.end.range.end.line,
            end_character: None,
            kind: Some(FoldingRangeKind::Comment),
            collapsed_text: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::{notifications::on_did_open_text_document, test_utils};

    use super::*;
    use async_lsp::lsp_types::{
        DidOpenTextDocumentParams, PartialResultParams, TextDocumentIdentifier, TextDocumentItem,
        WorkDoneProgressParams,
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
}
