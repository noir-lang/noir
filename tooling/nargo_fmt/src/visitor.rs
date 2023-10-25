mod expr;
mod item;
mod stmt;

use noirc_frontend::{hir::resolution::errors::Span, lexer::Lexer, token::Token};

use crate::{
    config::Config,
    utils::{self, FindToken},
};

pub(crate) struct FmtVisitor<'me> {
    config: &'me Config,
    buffer: String,
    pub(crate) source: &'me str,
    indent: Indent,
    last_position: u32,
}

impl<'me> FmtVisitor<'me> {
    pub(crate) fn new(source: &'me str, config: &'me Config) -> Self {
        Self {
            buffer: String::new(),
            config,
            source,
            last_position: 0,
            indent: Indent { block_indent: 0 },
        }
    }

    pub(crate) fn slice(&self, span: impl Into<Span>) -> &'me str {
        let span = span.into();
        &self.source[span.start() as usize..span.end() as usize]
    }

    fn span_after(&self, span: impl Into<Span>, token: Token) -> Span {
        let span = span.into();

        let slice = self.slice(span);
        let offset = slice.find_token(token).unwrap().end();

        (span.start() + offset..span.end()).into()
    }

    fn span_before(&self, span: impl Into<Span>, token: Token) -> Span {
        let span = span.into();

        let slice = self.slice(span);
        let offset = slice.find_token(token).unwrap().start();

        (span.start() + offset..span.end()).into()
    }

    fn shape(&self) -> Shape {
        Shape {
            width: self.config.max_width.saturating_sub(self.indent.width()),
            indent: self.indent,
        }
    }

    pub(crate) fn fork(&self) -> Self {
        Self {
            buffer: String::new(),
            config: self.config,
            source: self.source,
            last_position: self.last_position,
            indent: self.indent,
        }
    }

    pub(crate) fn finish(self) -> String {
        self.buffer
    }

    fn with_indent<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.indent.block_indent(self.config);
        let ret = f(self);
        self.indent.block_unindent(self.config);
        ret
    }

    fn at_start(&self) -> bool {
        self.buffer.is_empty()
    }

    fn push_str(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    #[track_caller]
    fn push_rewrite(&mut self, rewrite: String, span: Span) {
        let rewrite = utils::recover_comment_removed(self.slice(span), rewrite);
        self.format_missing_indent(span.start(), true);
        self.push_str(&rewrite);
    }

    fn format_missing(&mut self, end: u32) {
        self.format_missing_inner(end, |this, slice, _| this.push_str(slice));
    }

    #[track_caller]
    fn format_missing_indent(&mut self, end: u32, should_indent: bool) {
        self.format_missing_inner(end, |this, last_slice, slice| {
            this.push_str(last_slice.trim_end());

            if last_slice == slice && !this.at_start() {
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
            if !self.at_start() {
                if self.buffer.ends_with("{") {
                    self.push_str("\n");
                } else {
                    self.push_vertical_spaces(slice);
                }
            }

            let (result, last_end) = self.format_comment_in_block(slice);

            if result.is_empty() {
                process_last_slice(self, slice, slice);
            } else {
                self.push_str(&result.trim_end());
                let subslice = &slice[last_end as usize..];
                process_last_slice(self, subslice, subslice);
            }
        }
    }

    fn format_comment_in_block(&mut self, slice: &str) -> (String, u32) {
        let mut result = String::new();
        let mut last_end = 0;

        for spanned in Lexer::new(slice).skip_comments(false).flatten() {
            let span = spanned.to_span();
            last_end = span.end();

            if let Token::LineComment(_, _) | Token::BlockComment(_, _) = spanned.token() {
                result.push_str(&self.indent.to_string());
                result.push_str(&slice[span.start() as usize..span.end() as usize]);
                result.push('\n');
            }
        }

        (result, last_end)
    }

    fn push_vertical_spaces(&mut self, slice: &str) {
        let newline_upper_bound = 2;
        let newline_lower_bound = 1;

        let mut newline_count = bytecount::count(slice.as_bytes(), b'\n');
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
struct Indent {
    block_indent: usize,
}

impl Indent {
    fn width(&self) -> usize {
        self.block_indent
    }

    fn block_indent(&mut self, config: &Config) {
        self.block_indent += config.tab_spaces;
    }

    fn block_unindent(&mut self, config: &Config) {
        self.block_indent -= config.tab_spaces;
    }

    fn to_string_with_newline(self) -> String {
        "\n".to_string() + &self.to_string()
    }

    #[allow(clippy::inherent_to_string)]
    fn to_string(self) -> String {
        " ".repeat(self.block_indent)
    }
}

#[derive(Clone, Copy, Debug)]
struct Shape {
    width: usize,
    indent: Indent,
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum ExpressionType {
    Statement,
    SubExpression,
}
