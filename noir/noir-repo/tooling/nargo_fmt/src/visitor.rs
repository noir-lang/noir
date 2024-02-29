pub(crate) mod expr;
mod item;
mod stmt;

use noirc_frontend::{hir::resolution::errors::Span, lexer::Lexer, token::Token};

use crate::{
    config::Config,
    utils::{self, FindToken},
};

pub(crate) struct FmtVisitor<'me> {
    ignore_next_node: bool,
    pub(crate) config: &'me Config,
    buffer: String,
    pub(crate) source: &'me str,
    pub(crate) indent: Indent,
    last_position: u32,
}

impl<'me> FmtVisitor<'me> {
    pub(crate) fn new(source: &'me str, config: &'me Config) -> Self {
        Self {
            ignore_next_node: false,
            buffer: String::new(),
            config,
            source,
            last_position: 0,
            indent: Indent { block_indent: 0 },
        }
    }

    pub(crate) fn budget(&self, used_width: usize) -> usize {
        self.config.max_width.saturating_sub(used_width)
    }

    pub(crate) fn slice(&self, span: impl Into<Span>) -> &'me str {
        let span = span.into();
        &self.source[span.start() as usize..span.end() as usize]
    }

    pub(crate) fn span_after(&self, span: impl Into<Span>, token: Token) -> Span {
        let span = span.into();

        let slice = self.slice(span);
        let offset = slice.find_token(token).unwrap().end();

        (span.start() + offset..span.end()).into()
    }

    pub(crate) fn span_before(&self, span: impl Into<Span>, token: Token) -> Span {
        let span = span.into();

        let slice = self.slice(span);
        let offset = slice.find_token(token).unwrap().start();

        (span.start() + offset..span.end()).into()
    }

    pub(crate) fn shape(&self) -> Shape {
        Shape {
            width: self.config.max_width.saturating_sub(self.indent.width()),
            indent: self.indent,
        }
    }

    pub(crate) fn fork(&self) -> Self {
        Self {
            buffer: String::new(),
            ignore_next_node: self.ignore_next_node,
            config: self.config,
            source: self.source,
            last_position: self.last_position,
            indent: self.indent,
        }
    }

    pub(crate) fn finish(self) -> String {
        self.buffer
    }

    fn at_start(&self) -> bool {
        self.buffer.is_empty()
    }

    fn push_str(&mut self, s: &str) {
        let comments = Lexer::new(s).skip_comments(false).flatten().flat_map(|token| {
            if let Token::LineComment(content, _) | Token::BlockComment(content, _) =
                token.into_token()
            {
                let content = content.trim();
                content.strip_prefix("noir-fmt:").map(ToOwned::to_owned)
            } else {
                None
            }
        });

        for comment in comments {
            match comment.as_str() {
                "ignore" => self.ignore_next_node = true,
                this => unreachable!("unknown settings {this}"),
            }
        }

        self.buffer.push_str(s);
    }

    #[track_caller]
    fn push_rewrite(&mut self, rewrite: String, span: Span) {
        let original = self.slice(span);
        let changed_comment_content = utils::changed_comment_content(original, &rewrite);

        if changed_comment_content && self.config.error_on_lost_comment {
            panic!("not formatted because a comment would be lost: {rewrite:?}");
        }

        self.format_missing_indent(span.start(), true);

        let rewrite = if changed_comment_content || std::mem::take(&mut self.ignore_next_node) {
            original.to_string()
        } else {
            rewrite
        };

        self.push_str(&rewrite);

        if rewrite.starts_with('{') && rewrite.ends_with('}') {
            self.ignore_next_node = false;
        }
    }

    #[track_caller]
    fn format_missing_indent(&mut self, end: u32, should_indent: bool) {
        self.format_missing_inner(end, |this, last_slice, slice| {
            this.push_str(last_slice.trim_end());

            if (last_slice == slice && !this.at_start()) || this.buffer.ends_with("*/") {
                this.push_str("\n");
            }

            if should_indent {
                let indent = this.indent.to_string();
                this.push_str(&indent);
            }
        });
    }

    #[track_caller]
    fn format_missing_inner(
        &mut self,
        end: u32,
        process_last_slice: impl Fn(&mut Self, &str, &str),
    ) {
        let start = self.last_position;

        if start == end {
            if !self.at_start() {
                process_last_slice(self, "", "");
            }
            return;
        }

        let slice = self.slice(start..end);
        self.last_position = end;

        if slice.trim().is_empty() && !self.at_start() {
            self.push_vertical_spaces(slice);
            process_last_slice(self, "", slice);
        } else {
            let (result, last_end) = self.format_comment_in_block(slice);
            if result.trim().is_empty() {
                process_last_slice(self, slice, slice);
            } else {
                let last_snippet = &slice[last_end as usize..];
                self.push_str(&result);
                process_last_slice(self, last_snippet, &result);
            }
        }
    }

    pub(crate) fn format_comment_in_block(&mut self, slice: &str) -> (String, u32) {
        let mut result = String::new();
        let comments = Lexer::new(slice).skip_comments(false).skip_whitespaces(false).flatten();

        let indent = self.indent.to_string();
        for comment in comments {
            let span = comment.to_span();

            match comment.token() {
                Token::LineComment(_, _) | Token::BlockComment(_, _) => {
                    let comment = &slice[span.start() as usize..span.end() as usize];
                    if result.ends_with('\n') {
                        result.push_str(&indent);
                    } else if !self.at_start() {
                        result.push(' ');
                    }
                    result.push_str(comment);
                }
                Token::Whitespace(whitespaces) => {
                    let mut visitor = self.fork();
                    if whitespaces.contains('\n') {
                        visitor.push_vertical_spaces(whitespaces.trim_matches(' '));
                        result.push_str(&visitor.finish());
                    }
                }
                _ => {}
            }
        }

        (result, slice.len() as u32)
    }

    fn push_vertical_spaces(&mut self, slice: &str) {
        let newline_upper_bound = 2;
        let newline_lower_bound = 1;

        let mut newline_count = utils::count_newlines(slice);
        let offset = self.buffer.chars().rev().take_while(|c| *c == '\n').count();

        if newline_count + offset > newline_upper_bound {
            if offset >= newline_upper_bound {
                newline_count = 0;
            } else {
                newline_count = newline_upper_bound - offset;
            }
        } else if newline_count + offset < newline_lower_bound {
            if offset >= newline_lower_bound {
                newline_count = 0;
            } else {
                newline_count = newline_lower_bound - offset;
            }
        }

        let blank_lines = "\n".repeat(newline_count);
        self.push_str(&blank_lines);
    }

    pub(crate) fn format_comment(&self, span: Span) -> String {
        let slice = self.slice(span).trim();
        let pos = slice.find('/');

        if !slice.is_empty() && pos.is_some() {
            slice.to_string()
        } else {
            String::new()
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Indent {
    block_indent: usize,
}

impl Indent {
    pub(crate) fn width(&self) -> usize {
        self.block_indent
    }

    #[track_caller]
    pub(crate) fn block_indent(&mut self, config: &Config) {
        self.block_indent += config.tab_spaces;
    }

    #[track_caller]
    pub(crate) fn block_unindent(&mut self, config: &Config) {
        self.block_indent -= config.tab_spaces;
    }

    pub(crate) fn to_string_with_newline(self) -> String {
        "\n".to_string() + &self.to_string()
    }

    #[allow(clippy::inherent_to_string)]
    pub(crate) fn to_string(self) -> String {
        " ".repeat(self.block_indent)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Shape {
    pub(crate) width: usize,
    pub(crate) indent: Indent,
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum ExpressionType {
    Statement,
    SubExpression,
}
