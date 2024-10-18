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
mod chunks;
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
mod visibility;
mod where_clause;

pub(crate) struct Formatter<'a> {
    config: &'a Config,
    source: &'a str,
    lexer: Lexer<'a>,
    token: Token,
    token_span: Span,

    /// The current indentation level.
    indentation: usize,

    indentation_stack: Vec<usize>,

    /// How many characters we've written so far in the current line
    /// (useful to avoid exceeding the configurable maximum)
    current_line_width: usize,

    /// Whenever a comment is written, this flag is set to true.
    /// So, before formatting some chunk of code we can set this to false,
    /// format something and know if we wrote some comments.
    /// This is used, for example, when transforming `foo::{bar}` into `foo::bar`:
    /// we only do that if there were no comments between `{` and `}`.
    wrote_comment: bool,

    next_chunk_tag: usize,

    /// This is the buffer where we write the formatted code.
    pub(crate) buffer: String,
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
            current_line_width: 0,
            wrote_comment: false,
            next_chunk_tag: 0,
            buffer: String::new(),
        };
        formatter.bump();
        formatter
    }

    pub(crate) fn format_program(&mut self, parsed_module: ParsedModule) {
        self.skip_comments_and_whitespace_impl(
            false, // write lines
            true,  // at beginning
        );

        self.format_parsed_module(parsed_module);
    }

    fn format_parsed_module(&mut self, parsed_module: ParsedModule) {
        if !parsed_module.inner_doc_comments.is_empty() {
            self.format_inner_doc_comments();
        }

        for item in parsed_module.items {
            self.format_item(item);
        }
        self.write_line();
    }

    fn write_identifier(&mut self, ident: Ident) {
        self.skip_comments_and_whitespace();

        let Token::Ident(..) = self.token else {
            panic!("Expected identifier, got {:?}", self.token);
        };
        self.write(&ident.0.contents);
        self.bump();
    }

    fn write_identifier_or_integer(&mut self, ident: Ident) {
        self.skip_comments_and_whitespace();

        if !matches!(self.token, Token::Ident(..) | Token::Int(..)) {
            panic!("Expected identifier or integer, got {:?}", self.token);
        }
        self.write(&ident.0.contents);
        self.bump();
    }

    fn write_left_paren(&mut self) {
        self.write_token(Token::LeftParen);
    }

    fn write_right_paren(&mut self) {
        self.write_token(Token::RightParen);
    }

    fn write_left_brace(&mut self) {
        self.write_token(Token::LeftBrace);
    }

    fn write_right_brace(&mut self) {
        self.write_token(Token::RightBrace);
    }

    fn write_left_bracket(&mut self) {
        self.write_token(Token::LeftBracket);
    }

    fn write_right_bracket(&mut self) {
        self.write_token(Token::RightBracket);
    }

    fn write_comma(&mut self) {
        self.write_token(Token::Comma);
    }

    fn write_semicolon(&mut self) {
        self.write_token(Token::Semicolon);
    }

    fn write_keyword(&mut self, keyword: Keyword) {
        self.write_token(Token::Keyword(keyword));
    }

    fn write_token(&mut self, token: Token) {
        self.skip_comments_and_whitespace();
        if self.token == token {
            self.write_current_token();
            self.bump();
        } else {
            panic!("Expected token {:?}, got: {:?}", token, self.token);
        }
    }

    fn write_current_token(&mut self) {
        self.write(&self.token.to_string());
    }

    fn write_current_token_trimming_end(&mut self) {
        self.write(&self.token.to_string().trim_end());
    }

    fn write_current_token_as_in_source(&mut self) {
        self.write_source_span(self.token_span);
    }

    fn write_source_span(&mut self, span: Span) {
        self.write(&self.source[span.start() as usize..span.end() as usize]);
    }

    fn write_indentation(&mut self) {
        if self.buffer.ends_with(' ') {
            return;
        }

        for _ in 0..self.indentation {
            for _ in 0..self.config.tab_spaces {
                self.write(" ");
            }
        }
    }

    fn write(&mut self, str: &str) {
        self.buffer.push_str(str);

        if str.ends_with('\n') {
            self.current_line_width = 0;
        } else {
            self.current_line_width += str.chars().count();
        }
    }

    fn increase_indentation(&mut self) {
        self.indentation += 1;
    }

    fn decrease_indentation(&mut self) {
        self.indentation -= 1;
    }

    fn push_indentation(&mut self) {
        self.indentation_stack.push(self.indentation);
    }

    fn pop_indentation(&mut self) {
        self.indentation = self.indentation_stack.pop().unwrap();
    }

    fn bump(&mut self) -> Token {
        let next_token = self.read_token_internal();
        self.token_span = next_token.to_span();
        std::mem::replace(&mut self.token, next_token.into_token())
    }

    fn read_token_internal(&mut self) -> SpannedToken {
        loop {
            let token = self.lexer.next();
            if let Some(token) = token {
                match token {
                    Ok(token) => return token,
                    Err(err) => panic!("Expected lexer not to error, but got: {:?}", err),
                }
            } else {
                return SpannedToken::new(Token::EOF, Default::default());
            }
        }
    }
}
