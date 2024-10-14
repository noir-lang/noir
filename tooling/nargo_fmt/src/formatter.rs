use chunks::TextChunk;
use noirc_frontend::{
    ast::Ident,
    lexer::Lexer,
    token::{Keyword, Token},
    ParsedModule,
};

use crate::Config;

mod attribute;
mod chunks;
mod doc_comments;
mod function;
mod generics;
mod item;
mod module;
mod pattern;
mod structs;
mod types;
mod visibility;

pub(crate) struct SkipCommentsAndWhitespaceResult {
    pub(crate) wrote_comment: bool,
}

pub(crate) struct Formatter<'a> {
    config: &'a Config,
    lexer: Lexer<'a>,
    token: Token,
    indentation: usize,
    current_line_width: usize,
    pub(crate) buffer: String,
}

impl<'a> Formatter<'a> {
    pub(crate) fn new(source: &'a str, config: &'a Config) -> Self {
        let lexer = Lexer::new(source).skip_comments(false).skip_whitespaces(false);
        let mut formatter = Self {
            config,
            lexer,
            token: Token::EOF,
            indentation: 0,
            current_line_width: 0,
            buffer: String::new(),
        };
        formatter.bump();
        formatter
    }

    pub(crate) fn format_program(&mut self, parsed_module: ParsedModule) {
        self.format_parsed_module(parsed_module);
    }

    fn format_parsed_module(&mut self, parsed_module: ParsedModule) {
        for item in parsed_module.items {
            self.format_item(item);
        }
        self.write_line();
    }

    fn write_identifier(&mut self, ident: Ident) {
        // TODO: check that the ident matches
        let Token::Ident(..) = self.token else {
            panic!("Expected identifier, got {:?}", self.token);
        };
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

    fn write_space(&mut self) {
        self.skip_comments_and_whitespace();
        self.write_space_without_skipping_whitespace_and_comments();
    }

    fn write_space_without_skipping_whitespace_and_comments(&mut self) {
        if !self.buffer.ends_with('\n') && !self.buffer.ends_with(' ') {
            self.write(" ");
        }
    }

    fn skip_comments_and_whitespace(&mut self) -> SkipCommentsAndWhitespaceResult {
        self.skip_comments_and_whitespace_impl(
            false, // writing newline
        )
    }

    fn skip_comments_and_whitespace_impl(
        &mut self,
        writing_line: bool,
    ) -> SkipCommentsAndWhitespaceResult {
        let mut number_of_newlines = 0;
        let mut wrote_comment = false;
        loop {
            match &self.token {
                Token::Whitespace(whitespace) => {
                    number_of_newlines = whitespace.chars().filter(|char| *char == '\n').count();
                    self.bump();
                }
                Token::LineComment(_, None) => {
                    if number_of_newlines > 0 {
                        self.write_line_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else {
                        self.write_space_without_skipping_whitespace_and_comments();
                    }
                    self.write_current_token_trimming_end();
                    self.write_line_without_skipping_whitespace_and_comments();
                    number_of_newlines = 1;
                    self.bump();
                    wrote_comment = true;
                }
                Token::BlockComment(_, None) => {
                    if number_of_newlines > 0 {
                        self.write_line_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else {
                        self.write_space_without_skipping_whitespace_and_comments();
                    }
                    self.write_current_token();
                    self.bump();
                    wrote_comment = true;
                }
                _ => break,
            }
        }

        if number_of_newlines > 1 && writing_line {
            if self.buffer.ends_with("\n\n") {
                // Nothing
            } else if self.buffer.ends_with("\n") {
                self.write("\n")
            } else {
                self.write("\n\n");
            }
        }

        SkipCommentsAndWhitespaceResult { wrote_comment }
    }

    fn write_line(&mut self) {
        self.skip_comments_and_whitespace_impl(
            true, // writing newline
        );
        self.write_line_without_skipping_whitespace_and_comments();
    }

    fn write_line_without_skipping_whitespace_and_comments(&mut self) {
        if !self.buffer.ends_with('\n') {
            self.write("\n");
        }
    }

    fn write_indentation(&mut self) {
        for _ in 0..self.indentation {
            self.write("    ");
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

    fn bump(&mut self) -> Token {
        let next_token = self.read_token_internal();
        std::mem::replace(&mut self.token, next_token)
    }

    fn read_token_internal(&mut self) -> Token {
        loop {
            let token = self.lexer.next();
            if let Some(token) = token {
                match token {
                    Ok(token) => return token.into_token(),
                    Err(..) => panic!("Expected lexer not to error"),
                }
            } else {
                return Token::EOF;
            }
        }
    }
}
