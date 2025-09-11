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
        let mut lexer = Lexer::new(source, self.file_id).skip_comments(false);

        while let Some(token) = lexer.next() {
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
                        Self::push_line_comment_group(&group, &mut self.ranges);
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
