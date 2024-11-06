use buffer::Buffer;
use noirc_frontend::{
    ast::Ident,
    hir::resolution::errors::Span,
    lexer::Lexer,
    token::{Keyword, SpannedToken, Token},
    ParsedModule,
};

use crate::Config;

mod alias;
mod attribute;
mod buffer;
mod comments_and_whitespace;
mod doc_comments;
mod expression;
mod function;
mod generics;
mod global;
mod impls;
mod item;
mod lvalue;
mod module;
mod path;
mod pattern;
mod statement;
mod structs;
mod trait_impl;
mod traits;
mod type_expression;
mod types;
mod use_tree;
mod use_tree_merge;
mod visibility;
mod where_clause;

pub(crate) struct Formatter<'a> {
    pub(crate) config: &'a Config,
    source: &'a str,
    lexer: Lexer<'a>,
    token: Token,
    token_span: Span,

    /// The current indentation level.
    /// We allow it to be negative because in some cases we just want to decrease indentation
    /// to preemptively cancel out an indentation that will come later which we don't want to take effect,
    /// and we don't want to panic when reaching those negative values.
    pub(crate) indentation: i32,

    /// When formatting chunks we sometimes need to remember the current indentation
    /// and restore it later. This is what this stack is used for.
    indentation_stack: Vec<i32>,

    /// Whenever a comment is written, this counter is incremented.
    /// In this way we can know if comments were written while formatting some code:
    /// we remember the previous value, format, then see if it increased.
    /// This is used, for example, when transforming `foo::{bar}` into `foo::bar`:
    /// we only do that if there were no comments between `{` and `}`.
    written_comments_count: usize,

    /// If we find a comment like this one:
    ///
    /// // noir-fmt:ignore
    ///
    /// we won't format the next node (in some cases: only applies to statements and items).
    ignore_next: bool,

    /// A counter to create GroupTags.
    pub(crate) group_tag_counter: usize,

    /// We keep a copy of the config's max width because when we format chunk groups
    /// we somethings change this so that a group has less space to write to.
    pub(crate) max_width: usize,

    /// This is the buffer where we write the formatted code.
    pub(crate) buffer: Buffer,
}

impl<'a> Formatter<'a> {
    pub(crate) fn new(source: &'a str, config: &'a Config) -> Self {
        let lexer = Lexer::new(source).skip_comments(false).skip_whitespaces(false);
        let mut formatter = Self {
            config,
            source,
            lexer,
            token: Token::EOF,
            token_span: Default::default(),
            indentation: 0,
            indentation_stack: Vec::new(),
            written_comments_count: 0,
            ignore_next: false,
            group_tag_counter: 0,
            max_width: config.max_width,
            buffer: Buffer::default(),
        };
        formatter.bump();
        formatter
    }

    pub(crate) fn format_program(&mut self, parsed_module: ParsedModule) {
        self.skip_whitespace();
        self.skip_comments_and_whitespace_impl(
            true, // write lines
            true, // at beginning
        );

        self.format_parsed_module(parsed_module, self.ignore_next);
    }

    pub(crate) fn format_parsed_module(&mut self, parsed_module: ParsedModule, ignore_next: bool) {
        if !parsed_module.inner_doc_comments.is_empty() {
            self.format_inner_doc_comments();
        }

        self.format_items(parsed_module.items, ignore_next);
        self.write_line();
    }

    pub(crate) fn write_identifier(&mut self, ident: Ident) {
        self.skip_comments_and_whitespace();

        let Token::Ident(..) = self.token else {
            panic!("Expected identifier, got {:?}", self.token);
        };
        self.write(&ident.0.contents);
        self.bump();
    }

    pub(crate) fn write_identifier_or_integer(&mut self, ident: Ident) {
        self.skip_comments_and_whitespace();

        if !matches!(self.token, Token::Ident(..) | Token::Int(..)) {
            panic!("Expected identifier or integer, got {:?}", self.token);
        }
        self.write(&ident.0.contents);
        self.bump();
    }

    pub(crate) fn write_left_paren(&mut self) {
        self.write_token(Token::LeftParen);
    }

    pub(crate) fn write_right_paren(&mut self) {
        self.write_token(Token::RightParen);
    }

    pub(crate) fn write_left_brace(&mut self) {
        self.write_token(Token::LeftBrace);
    }

    pub(crate) fn write_right_brace(&mut self) {
        self.write_token(Token::RightBrace);
    }

    pub(crate) fn write_left_bracket(&mut self) {
        self.write_token(Token::LeftBracket);
    }

    pub(crate) fn write_right_bracket(&mut self) {
        self.write_token(Token::RightBracket);
    }

    pub(crate) fn write_comma(&mut self) {
        self.write_token(Token::Comma);
    }

    pub(crate) fn write_semicolon(&mut self) {
        self.write_token(Token::Semicolon);
    }

    /// Writes the given keyword, if the current token is that keyword
    /// (so this is a check that we are producing a token we expect to be in the source
    /// we are traversing). Then advances to the next token.
    ///
    /// Calls `write_token` so comments and whitespaces are skipped before writing the keyword.
    pub(crate) fn write_keyword(&mut self, keyword: Keyword) {
        self.write_token(Token::Keyword(keyword));
    }

    /// Writes the given token, if the current token is the same as the given one
    /// (so this is a check that we are producing a token we expect to be in the source
    /// we are traversing). Then advances to the next token.
    ///
    /// Before writing the token any comments and spaces are skipped. This is so that
    /// a caller can call `write_token`, `write_keyword`, `write_space`, etc., without
    /// having to explicitly call `skip_comments_and_whitespace` in between those calls.
    pub(crate) fn write_token(&mut self, token: Token) {
        self.skip_comments_and_whitespace();
        if self.token == token {
            self.write_current_token_and_bump();
        } else {
            panic!("Expected token {:?}, got: {:?}", token, self.token);
        }
    }

    /// Writes the current token but doesn't advance to the next one.
    pub(crate) fn write_current_token(&mut self) {
        self.write(&self.token.to_string());
    }

    /// Writes the current token and advances to the next one
    pub(crate) fn write_current_token_and_bump(&mut self) {
        self.write(&self.token.to_string());
        self.bump();
    }

    /// Writes the current token trimming its end but doesn't advance to the next one.
    /// Mainly used when writing comment lines, because we never want trailing spaces
    /// inside comments.
    pub(crate) fn write_current_token_trimming_end(&mut self) {
        self.write(self.token.to_string().trim_end());
    }

    /// Writes the current token but without turning it into a string using `to_string()`.
    /// Instead, we check the token's span and format what's in the original source there
    /// (useful when formatting integer tokens, because a token like 0xFF ends up being an
    /// integer with a value 255, but we don't want to change 0xFF to 255).
    pub(crate) fn write_current_token_as_in_source(&mut self) {
        self.write_source_span(self.token_span);
    }

    /// Writes whatever is in the given span relative to the file's source that's being formatted.
    pub(crate) fn write_source_span(&mut self, span: Span) {
        self.write(&self.source[span.start() as usize..span.end() as usize]);
    }

    /// Writes the current indentation to the buffer, but only if the buffer
    /// is empty or it ends with a newline (otherwise we'd be indenting when not needed).
    pub(crate) fn write_indentation(&mut self) {
        if !(self.buffer.is_empty() || self.buffer.ends_with_newline()) {
            return;
        }

        for _ in 0..self.indentation {
            for _ in 0..self.config.tab_spaces {
                self.write(" ");
            }
        }
    }

    /// Writes whatever is in the source at the given span without formatting it,
    /// then advances the lexer until past the end of the span.
    /// This is mainly used to avoid formatting items and statements when a
    /// `noir-fmt:ignore` comment is found.
    pub(super) fn write_and_skip_span_without_formatting(&mut self, span: Span) {
        self.write_source_span(span);

        while self.token_span.start() < span.end() && self.token != Token::EOF {
            self.bump();
        }
    }

    /// Advances the lexer until past the given span end without writing anything to the buffer.
    pub(super) fn skip_past_span_end_without_formatting(&mut self, span_end: u32) {
        while self.token_span.start() < span_end && self.token != Token::EOF {
            self.bump();
        }
    }

    /// Writes a string to the buffer.
    pub(crate) fn write(&mut self, str: &str) {
        self.buffer.write(str);
    }

    pub(crate) fn current_line_width(&self) -> usize {
        self.buffer.current_line_width()
    }

    pub(crate) fn increase_indentation(&mut self) {
        self.indentation += 1;
    }

    pub(crate) fn decrease_indentation(&mut self) {
        self.indentation -= 1;
    }

    pub(crate) fn push_indentation(&mut self) {
        self.indentation_stack.push(self.indentation);
    }

    pub(crate) fn pop_indentation(&mut self) {
        self.indentation = self.indentation_stack.pop().unwrap();
    }

    pub(crate) fn is_at_keyword(&self, keyword: Keyword) -> bool {
        self.is_at(Token::Keyword(keyword))
    }

    pub(crate) fn is_at(&self, token: Token) -> bool {
        self.token == token
    }

    /// Advances to the next token (the current token is not written).
    pub(crate) fn bump(&mut self) -> Token {
        self.ignore_next = false;

        let next_token = self.read_token_internal();
        self.token_span = next_token.to_span();
        std::mem::replace(&mut self.token, next_token.into_token())
    }

    pub(crate) fn read_token_internal(&mut self) -> SpannedToken {
        let token = self.lexer.next();
        if let Some(token) = token {
            match token {
                Ok(token) => token,
                Err(err) => panic!("Expected lexer not to error, but got: {:?}", err),
            }
        } else {
            SpannedToken::new(Token::EOF, Default::default())
        }
    }
}
