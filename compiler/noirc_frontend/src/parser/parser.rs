use acvm::FieldElement;
use noirc_errors::Span;

use crate::{
    ast::{Ident, LValue},
    lexer::{Lexer, SpannedTokenResult},
    token::{IntType, Keyword, SpannedToken, Token, TokenKind, Tokens},
};

use super::{ParsedModule, ParserError, ParserErrorReason};

mod attributes;
mod call;
mod doc_comments;
mod expression;
mod function;
mod generics;
mod global;
mod impls;
mod infix;
mod item;
mod item_visibility;
mod lambda;
mod modifiers;
mod module;
mod path;
mod pattern;
mod statement;
mod structs;
mod tests;
mod traits;
mod type_alias;
mod type_expression;
mod types;
mod use_tree;
mod where_clause;

/// Entry function for the parser - also handles lexing internally.
///
/// Given a source_program string, return the ParsedModule Ast representation
/// of the program along with any parsing errors encountered. If the parsing errors
/// Vec is non-empty, there may be Error nodes in the Ast to fill in the gaps that
/// failed to parse. Otherwise the Ast is guaranteed to have 0 Error nodes.
pub fn parse_program(source_program: &str) -> (ParsedModule, Vec<ParserError>) {
    let lexer = Lexer::new(source_program);
    let mut parser = Parser::for_lexer(lexer);
    let program = parser.parse_module();
    let errors = parser.errors;
    (program, errors)
}

pub fn parse_result<'a, T, F>(mut parser: Parser<'a>, f: F) -> Result<T, Vec<ParserError>>
where
    F: FnOnce(&mut Parser<'a>) -> T,
{
    let item = f(&mut parser);
    if parser.errors.is_empty() {
        Ok(item)
    } else {
        Err(parser.errors)
    }
}

enum TokenStream<'a> {
    Lexer(Lexer<'a>),
    Tokens(Tokens),
}

impl<'a> TokenStream<'a> {
    fn next(&mut self) -> Option<SpannedTokenResult> {
        match self {
            TokenStream::Lexer(lexer) => lexer.next(),
            TokenStream::Tokens(tokens) => tokens.0.pop().map(Ok),
        }
    }
}

pub struct Parser<'a> {
    errors: Vec<ParserError>,
    tokens: TokenStream<'a>,

    // We always have one look-ahead token to see if we get `&mut` or just `&`
    // (`&` and `mut` are two separate tokens)
    token: SpannedToken,
    next_token: SpannedToken,

    current_token_span: Span,
    previous_token_span: Span,
}

impl<'a> Parser<'a> {
    pub fn for_lexer(lexer: Lexer<'a>) -> Self {
        Self::new(TokenStream::Lexer(lexer))
    }

    pub fn for_tokens(mut tokens: Tokens) -> Self {
        tokens.0.reverse();
        Self::new(TokenStream::Tokens(tokens))
    }

    pub fn for_str(str: &'a str) -> Self {
        Self::for_lexer(Lexer::new(str))
    }

    fn new(tokens: TokenStream<'a>) -> Self {
        let mut parser = Self {
            errors: Vec::new(),
            tokens,
            token: SpannedToken::default(),
            next_token: SpannedToken::default(),
            current_token_span: Default::default(),
            previous_token_span: Default::default(),
        };
        parser.read_two_first_tokens();
        parser
    }

    pub(crate) fn parse_module(&mut self) -> ParsedModule {
        let inner_doc_comments = self.parse_inner_doc_comments();
        let items = self.parse_items();

        ParsedModule { items, inner_doc_comments }
    }

    pub(crate) fn parse_lvalue(&mut self) -> LValue {
        todo!("Parser")
    }

    fn next_token(&mut self) {
        self.previous_token_span = self.current_token_span;
        let token = self.read_token_internal();
        let next_token = std::mem::take(&mut self.next_token);
        self.token = next_token;
        self.next_token = token;
        self.current_token_span = self.token.to_span();
    }

    fn read_two_first_tokens(&mut self) {
        self.token = self.read_token_internal();
        self.current_token_span = self.token.to_span();
        self.next_token = self.read_token_internal();
    }

    fn read_token_internal(&mut self) -> SpannedToken {
        loop {
            let token = self.tokens.next();
            if let Some(token) = token {
                match token {
                    Ok(token) => return token,
                    Err(lexer_error) => self.errors.push(lexer_error.into()),
                }
            } else {
                return SpannedToken::default();
            }
        }
    }

    fn eat_kind(&mut self, kind: TokenKind) -> Option<SpannedToken> {
        if self.token.kind() == kind {
            let token = std::mem::take(&mut self.token);
            self.next_token();
            Some(token)
        } else {
            None
        }
    }

    fn eat_keyword(&mut self, keyword: Keyword) -> bool {
        if let Token::Keyword(kw) = self.token.token() {
            if *kw == keyword {
                self.next_token();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn eat_ident(&mut self) -> Option<Ident> {
        if let Some(token) = self.eat_kind(TokenKind::Ident) {
            match token.into_token() {
                Token::Ident(ident) => Some(Ident::new(ident, self.previous_token_span)),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    fn eat_self(&mut self) -> bool {
        if let Token::Ident(ident) = self.token.token() {
            if ident == "self" {
                self.next_token();
                return true;
            }
        }

        false
    }

    fn eat_int_type(&mut self) -> Option<IntType> {
        let is_int_type = matches!(self.token.token(), Token::IntType(..));
        if is_int_type {
            let token = std::mem::take(&mut self.token);
            self.next_token();
            match token.into_token() {
                Token::IntType(int_type) => Some(int_type),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    fn eat_int(&mut self) -> Option<FieldElement> {
        if matches!(self.token.token(), Token::Int(..)) {
            let token = std::mem::take(&mut self.token);
            self.next_token();
            match token.into_token() {
                Token::Int(int) => Some(int),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    fn eat_bool(&mut self) -> Option<bool> {
        if matches!(self.token.token(), Token::Bool(..)) {
            let token = std::mem::take(&mut self.token);
            self.next_token();
            match token.into_token() {
                Token::Bool(bool) => Some(bool),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    fn eat_str(&mut self) -> Option<String> {
        if matches!(self.token.token(), Token::Str(..)) {
            let token = std::mem::take(&mut self.token);
            self.next_token();
            match token.into_token() {
                Token::Str(string) => Some(string),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    fn eat_raw_str(&mut self) -> Option<(String, u8)> {
        if matches!(self.token.token(), Token::RawStr(..)) {
            let token = std::mem::take(&mut self.token);
            self.next_token();
            match token.into_token() {
                Token::RawStr(string, n) => Some((string, n)),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    fn eat_fmt_str(&mut self) -> Option<String> {
        if matches!(self.token.token(), Token::FmtStr(..)) {
            let token = std::mem::take(&mut self.token);
            self.next_token();
            match token.into_token() {
                Token::FmtStr(string) => Some(string),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    fn eat_quote(&mut self) -> Option<Tokens> {
        if matches!(self.token.token(), Token::Quote(..)) {
            let token = std::mem::take(&mut self.token);
            self.next_token();
            match token.into_token() {
                Token::Quote(tokens) => Some(tokens),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    fn eat_comma(&mut self) -> bool {
        self.eat(Token::Comma)
    }

    fn eat_commas(&mut self) -> bool {
        if self.eat_comma() {
            while self.eat_comma() {
                self.push_error(ParserErrorReason::UnexpectedComma, self.previous_token_span);
            }
            true
        } else {
            false
        }
    }

    fn eat_semicolon(&mut self) -> bool {
        self.eat(Token::Semicolon)
    }

    fn eat_semicolons(&mut self) -> bool {
        if self.eat_semicolon() {
            while self.eat_semicolon() {
                self.push_error(ParserErrorReason::UnexpectedSemicolon, self.previous_token_span);
            }
            true
        } else {
            false
        }
    }

    fn eat_colon(&mut self) -> bool {
        self.eat(Token::Colon)
    }

    fn eat_double_colon(&mut self) -> bool {
        self.eat(Token::DoubleColon)
    }

    fn eat_left_paren(&mut self) -> bool {
        self.eat(Token::LeftParen)
    }

    fn eat_right_paren(&mut self) -> bool {
        self.eat(Token::RightParen)
    }

    fn eat_left_brace(&mut self) -> bool {
        self.eat(Token::LeftBrace)
    }

    fn eat_right_brace(&mut self) -> bool {
        self.eat(Token::RightBrace)
    }

    fn eat_left_bracket(&mut self) -> bool {
        self.eat(Token::LeftBracket)
    }

    fn eat_right_bracket(&mut self) -> bool {
        self.eat(Token::RightBracket)
    }

    fn eat_less(&mut self) -> bool {
        self.eat(Token::Less)
    }

    fn eat_greater(&mut self) -> bool {
        self.eat(Token::Greater)
    }

    fn eat_assign(&mut self) -> bool {
        self.eat(Token::Assign)
    }

    fn eat_plus(&mut self) -> bool {
        self.eat(Token::Plus)
    }

    fn eat_dot(&mut self) -> bool {
        self.eat(Token::Dot)
    }

    fn eat_pipe(&mut self) -> bool {
        self.eat(Token::Pipe)
    }

    fn eat(&mut self, token: Token) -> bool {
        if self.token.token() == &token {
            self.next_token();
            true
        } else {
            false
        }
    }

    fn is_eof(&self) -> bool {
        self.token.token() == &Token::EOF
    }

    fn span_since(&self, start_span: Span) -> Span {
        if self.current_token_span == start_span {
            start_span
        } else {
            let end_span = self.previous_token_span;
            Span::from(start_span.start()..end_span.end())
        }
    }

    fn push_error(&mut self, reason: ParserErrorReason, span: Span) {
        self.errors.push(ParserError::with_reason(reason, span));
    }
}
