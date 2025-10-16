use async_lsp::lsp_types::{self, FoldingRange, FoldingRangeKind};
use fm::{FileId, FileMap};
use noirc_frontend::{
    lexer::Lexer,
    token::{DocStyle, Token},
};

use crate::requests::to_lsp_location;

pub(super) struct CommentsCollector<'files> {
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

impl<'files> CommentsCollector<'files> {
    pub(super) fn new(file_id: FileId, files: &'files FileMap) -> Self {
        Self { file_id, files, ranges: Vec::new(), current_line_comment_group: None }
    }

    pub(super) fn collect(mut self, source: &str) -> Vec<FoldingRange> {
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
                            && location.range.start.line - group.end.range.end.line <= 1
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
        let start_line = group.start.range.start.line;
        let end_line = group.end.range.end.line;
        if start_line == end_line {
            return;
        }

        ranges.push(FoldingRange {
            start_line,
            start_character: None,
            end_line,
            end_character: None,
            kind: Some(FoldingRangeKind::Comment),
            collapsed_text: None,
        });
    }
}
