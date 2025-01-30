use acvm::FieldElement;
use modifiers::Modifiers;
use noirc_errors::Span;

use crate::{
    ast::{Ident, ItemVisibility},
    lexer::{Lexer, SpannedTokenResult},
    token::{FmtStrFragment, IntType, Keyword, SpannedToken, Token, TokenKind, Tokens},
};

use super::{labels::ParsingRuleLabel, ParsedModule, ParserError, ParserErrorReason};

mod arguments;
mod attributes;
mod doc_comments;
mod enums;
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
mod parse_many;
mod path;
mod pattern;
mod statement;
mod statement_or_expression_or_lvalue;
mod structs;
mod tests;
mod traits;
mod type_alias;
mod type_expression;
mod types;
mod use_tree;
mod where_clause;

pub use statement_or_expression_or_lvalue::StatementOrExpressionOrLValue;

/// Entry function for the parser - also handles lexing internally.
///
/// Given a source_program string, return the ParsedModule Ast representation
/// of the program along with any parsing errors encountered. If the parsing errors
/// Vec is non-empty, there may be Error nodes in the Ast to fill in the gaps that
/// failed to parse. Otherwise the Ast is guaranteed to have 0 Error nodes.
pub fn parse_program(source_program: &str) -> (ParsedModule, Vec<ParserError>) {
    let lexer = Lexer::new(source_program);
    let mut parser = Parser::for_lexer(lexer);
    let program = parser.parse_program();
    let errors = parser.errors;
    (program, errors)
}

enum TokenStream<'a> {
    Lexer(Lexer<'a>),
    Tokens(Tokens),
}

impl<'a> TokenStream<'a> {
    fn next(&mut self) -> Option<SpannedTokenResult> {
        match self {
            TokenStream::Lexer(lexer) => lexer.next(),
            TokenStream::Tokens(tokens) => {
                // NOTE: `TokenStream::Tokens` is only created via `Parser::for_tokens(tokens)` which
                // reverses `tokens`. That's why using `pop` here is fine (done for performance reasons).
                tokens.0.pop().map(Ok)
            }
        }
    }
}

pub struct Parser<'a> {
    pub(crate) errors: Vec<ParserError>,
    tokens: TokenStream<'a>,

    // We always have one look-ahead token for these cases:
    // - check if we get `&` or `&mut`
    // - check if we get `>` or `>>`
    token: SpannedToken,
    next_token: SpannedToken,
    current_token_span: Span,
    previous_token_span: Span,

    /// The current statement's doc comments.
    /// This is used to eventually know if an `unsafe { ... }` expression is documented
    /// in its containing statement. For example:
    ///
    /// ```noir
    /// /// Safety: test
    /// let x = unsafe { call() };
    /// ```
    statement_doc_comments: Option<StatementDocComments>,
}

#[derive(Debug)]
pub(crate) struct StatementDocComments {
    pub(crate) doc_comments: Vec<String>,
    pub(crate) start_span: Span,
    pub(crate) end_span: Span,

    /// Were these doc comments "read" by an unsafe statement?
    /// If not, these doc comments aren't documenting anything and they produce an error.
    pub(crate) read: bool,
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
            token: eof_spanned_token(),
            next_token: eof_spanned_token(),
            current_token_span: Default::default(),
            previous_token_span: Default::default(),
            statement_doc_comments: None,
        };
        parser.read_two_first_tokens();
        parser
    }

    /// Program = Module
    pub(crate) fn parse_program(&mut self) -> ParsedModule {
        self.parse_module(
            false, // nested
        )
    }

    /// Module = InnerDocComments Item*
    pub(crate) fn parse_module(&mut self, nested: bool) -> ParsedModule {
        let inner_doc_comments = self.parse_inner_doc_comments();
        let items = self.parse_module_items(nested);

        ParsedModule { items, inner_doc_comments }
    }

    /// Invokes `parsing_function` (`parsing_function` must be some `parse_*` method of the parser)
    /// and returns the result (and any warnings) if the parser has no errors, and if the parser consumed all tokens.
    /// Otherwise returns the list of errors.
    pub fn parse_result<T, F>(
        mut self,
        parsing_function: F,
    ) -> Result<(T, Vec<ParserError>), Vec<ParserError>>
    where
        F: FnOnce(&mut Parser<'a>) -> T,
    {
        let item = parsing_function(&mut self);
        if !self.at_eof() {
            self.expected_token(Token::EOF);
            return Err(self.errors);
        }

        let all_warnings = self.errors.iter().all(|error| error.is_warning());
        if all_warnings {
            Ok((item, self.errors))
        } else {
            Err(self.errors)
        }
    }

    /// Bumps this parser by one token. Returns the token that was previously the "current" token.
    fn bump(&mut self) -> SpannedToken {
        self.previous_token_span = self.current_token_span;
        let next_next_token = self.read_token_internal();
        let next_token = std::mem::replace(&mut self.next_token, next_next_token);
        let token = std::mem::replace(&mut self.token, next_token);
        self.current_token_span = self.token.to_span();
        token
    }

    fn read_two_first_tokens(&mut self) {
        self.token = self.read_token_internal();
        self.current_token_span = self.token.to_span();
        self.next_token = self.read_token_internal();
    }

    fn read_token_internal(&mut self) -> SpannedToken {
        loop {
            match self.tokens.next() {
                Some(Ok(token)) => return token,
                Some(Err(lexer_error)) => self.errors.push(lexer_error.into()),
                None => return eof_spanned_token(),
            }
        }
    }

    fn eat_kind(&mut self, kind: TokenKind) -> Option<SpannedToken> {
        if self.token.kind() == kind {
            Some(self.bump())
        } else {
            None
        }
    }

    fn eat_keyword(&mut self, keyword: Keyword) -> bool {
        if let Token::Keyword(kw) = self.token.token() {
            if *kw == keyword {
                self.bump();
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
                self.bump();
                return true;
            }
        }

        false
    }

    fn eat_int_type(&mut self) -> Option<IntType> {
        let is_int_type = matches!(self.token.token(), Token::IntType(..));
        if is_int_type {
            let token = self.bump();
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
            let token = self.bump();
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
            let token = self.bump();
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
            let token = self.bump();
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
            let token = self.bump();
            match token.into_token() {
                Token::RawStr(string, n) => Some((string, n)),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    fn eat_fmt_str(&mut self) -> Option<(Vec<FmtStrFragment>, u32)> {
        if matches!(self.token.token(), Token::FmtStr(..)) {
            let token = self.bump();
            match token.into_token() {
                Token::FmtStr(fragments, length) => Some((fragments, length)),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    fn eat_quote(&mut self) -> Option<Tokens> {
        if matches!(self.token.token(), Token::Quote(..)) {
            let token = self.bump();
            match token.into_token() {
                Token::Quote(tokens) => Some(tokens),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    fn eat_attribute_start(&mut self) -> Option<bool> {
        if matches!(self.token.token(), Token::AttributeStart { is_inner: false, .. }) {
            let token = self.bump();
            match token.into_token() {
                Token::AttributeStart { is_tag, .. } => Some(is_tag),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    fn eat_inner_attribute_start(&mut self) -> Option<bool> {
        if matches!(self.token.token(), Token::AttributeStart { is_inner: true, .. }) {
            let token = self.bump();
            match token.into_token() {
                Token::AttributeStart { is_tag, .. } => Some(is_tag),
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

    fn eat_left_bracket(&mut self) -> bool {
        self.eat(Token::LeftBracket)
    }

    fn eat_right_bracket(&mut self) -> bool {
        self.eat(Token::RightBracket)
    }

    fn eat_less(&mut self) -> bool {
        self.eat(Token::Less)
    }

    fn eat_assign(&mut self) -> bool {
        self.eat(Token::Assign)
    }

    fn eat_dot(&mut self) -> bool {
        self.eat(Token::Dot)
    }

    fn eat_pipe(&mut self) -> bool {
        self.eat(Token::Pipe)
    }

    fn eat(&mut self, token: Token) -> bool {
        if self.token.token() == &token {
            self.bump();
            true
        } else {
            false
        }
    }

    fn eat_keyword_or_error(&mut self, keyword: Keyword) {
        if !self.eat_keyword(keyword) {
            self.expected_token(Token::Keyword(keyword));
        }
    }

    fn eat_or_error(&mut self, token: Token) {
        if !self.eat(token.clone()) {
            self.expected_token(token);
        }
    }

    fn at(&self, token: Token) -> bool {
        self.token.token() == &token
    }

    fn at_keyword(&self, keyword: Keyword) -> bool {
        self.at(Token::Keyword(keyword))
    }

    fn next_is(&self, token: Token) -> bool {
        self.next_token.token() == &token
    }

    fn at_eof(&self) -> bool {
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

    fn span_at_previous_token_end(&self) -> Span {
        Span::from(self.previous_token_span.end()..self.previous_token_span.end())
    }

    fn expected_identifier(&mut self) {
        self.expected_label(ParsingRuleLabel::Identifier);
    }

    fn expected_token(&mut self, token: Token) {
        self.errors.push(ParserError::expected_token(
            token,
            self.token.token().clone(),
            self.current_token_span,
        ));
    }

    fn expected_one_of_tokens(&mut self, tokens: &[Token]) {
        self.errors.push(ParserError::expected_one_of_tokens(
            tokens,
            self.token.token().clone(),
            self.current_token_span,
        ));
    }

    fn expected_label(&mut self, label: ParsingRuleLabel) {
        self.errors.push(ParserError::expected_label(
            label,
            self.token.token().clone(),
            self.current_token_span,
        ));
    }

    fn expected_token_separating_items(&mut self, token: Token, items: &'static str, span: Span) {
        self.push_error(ParserErrorReason::ExpectedTokenSeparatingTwoItems { token, items }, span);
    }

    fn expected_mut_after_ampersand(&mut self) {
        self.push_error(
            ParserErrorReason::ExpectedMutAfterAmpersand { found: self.token.token().clone() },
            self.current_token_span,
        );
    }

    fn modifiers_not_followed_by_an_item(&mut self, modifiers: Modifiers) {
        self.visibility_not_followed_by_an_item(modifiers);
        self.unconstrained_not_followed_by_an_item(modifiers);
        self.comptime_not_followed_by_an_item(modifiers);
    }

    fn visibility_not_followed_by_an_item(&mut self, modifiers: Modifiers) {
        if modifiers.visibility != ItemVisibility::Private {
            self.push_error(
                ParserErrorReason::VisibilityNotFollowedByAnItem {
                    visibility: modifiers.visibility,
                },
                modifiers.visibility_span,
            );
        }
    }

    fn unconstrained_not_followed_by_an_item(&mut self, modifiers: Modifiers) {
        if let Some(span) = modifiers.unconstrained {
            self.push_error(ParserErrorReason::UnconstrainedNotFollowedByAnItem, span);
        }
    }

    fn comptime_not_followed_by_an_item(&mut self, modifiers: Modifiers) {
        if let Some(span) = modifiers.comptime {
            self.push_error(ParserErrorReason::ComptimeNotFollowedByAnItem, span);
        }
    }

    fn comptime_mutable_and_unconstrained_not_applicable(&mut self, modifiers: Modifiers) {
        self.mutable_not_applicable(modifiers);
        self.comptime_not_applicable(modifiers);
        self.unconstrained_not_applicable(modifiers);
    }

    fn mutable_not_applicable(&mut self, modifiers: Modifiers) {
        if let Some(span) = modifiers.mutable {
            self.push_error(ParserErrorReason::MutableNotApplicable, span);
        }
    }

    fn comptime_not_applicable(&mut self, modifiers: Modifiers) {
        if let Some(span) = modifiers.comptime {
            self.push_error(ParserErrorReason::ComptimeNotApplicable, span);
        }
    }

    fn unconstrained_not_applicable(&mut self, modifiers: Modifiers) {
        if let Some(span) = modifiers.unconstrained {
            self.push_error(ParserErrorReason::UnconstrainedNotApplicable, span);
        }
    }

    fn push_error(&mut self, reason: ParserErrorReason, span: Span) {
        self.errors.push(ParserError::with_reason(reason, span));
    }
}

fn eof_spanned_token() -> SpannedToken {
    SpannedToken::new(Token::EOF, Default::default())
}
