use acvm::FieldElement;
use fm::FileId;
use modifiers::Modifiers;
use noirc_errors::{Location, Span};

use crate::{
    ast::{Ident, ItemVisibility},
    lexer::{Lexer, lexer::LocatedTokenResult},
    node_interner::ExprId,
    token::{FmtStrFragment, IntegerTypeSuffix, Keyword, LocatedToken, Token, TokenKind, Tokens},
};

use super::{ParsedModule, ParserError, ParserErrorReason, labels::ParsingRuleLabel};

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
pub fn parse_program(source_program: &str, file_id: FileId) -> (ParsedModule, Vec<ParserError>) {
    let lexer = Lexer::new(source_program, file_id);
    let mut parser = Parser::for_lexer(lexer);
    let program = parser.parse_program();
    let errors = parser.errors;
    (program, errors)
}

pub fn parse_program_with_dummy_file(source_program: &str) -> (ParsedModule, Vec<ParserError>) {
    parse_program(source_program, FileId::dummy())
}

enum TokenStream<'a> {
    Lexer(Lexer<'a>),
    Tokens(Tokens),
}

impl TokenStream<'_> {
    fn next(&mut self) -> Option<LocatedTokenResult> {
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

/// Maximum recursion depth for parsing nested expressions.
/// This limit prevents stack overflow when parsing deeply nested expressions.
pub(super) const MAX_PARSER_RECURSION_DEPTH: u32 = 100;

pub struct Parser<'a> {
    pub(crate) errors: Vec<ParserError>,
    tokens: TokenStream<'a>,

    // We always have one look-ahead token for these cases:
    // - check if we get `&` or `&mut`
    // - check if we get `>` or `>>`
    token: LocatedToken,
    next_token: LocatedToken,
    current_token_location: Location,
    previous_token_location: Location,

    // We also keep track of comments that appear right before a token,
    // because `unsafe { }` requires one before it.
    current_token_comments: String,
    next_token_comments: String,

    /// The current statement's comments.
    /// This is used to eventually know if an `unsafe { ... }` expression is commented
    /// in its containing statement. For example:
    ///
    /// ```noir
    /// // Safety: test
    /// let x = unsafe { call() };
    /// ```
    statement_comments: Option<String>,

    /// Current recursion depth for parsing nested expressions.
    /// Used to prevent stack overflow from deeply nested expressions.
    recursion_depth: u32,

    /// Set to true when recovering from a recursion depth overflow.
    /// Used to suppress cascading errors during stack unwinding.
    recovering_from_depth_overflow: bool,
}

impl<'a> Parser<'a> {
    pub fn for_lexer(lexer: Lexer<'a>) -> Self {
        Self::new(TokenStream::Lexer(lexer.skip_comments(false)))
    }

    pub fn for_tokens(mut tokens: Tokens) -> Self {
        tokens.0.reverse();
        Self::new(TokenStream::Tokens(tokens))
    }

    pub fn for_str(str: &'a str, file_id: FileId) -> Self {
        Self::for_lexer(Lexer::new(str, file_id))
    }

    pub fn for_str_with_dummy_file(str: &'a str) -> Self {
        Self::for_str(str, FileId::dummy())
    }

    fn new(tokens: TokenStream<'a>) -> Self {
        let mut parser = Self {
            errors: Vec::new(),
            tokens,
            token: eof_located_token(),
            next_token: eof_located_token(),
            current_token_location: Location::dummy(),
            previous_token_location: Location::dummy(),
            current_token_comments: String::new(),
            next_token_comments: String::new(),
            statement_comments: None,
            recursion_depth: 0,
            recovering_from_depth_overflow: false,
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
        if all_warnings { Ok((item, self.errors)) } else { Err(self.errors) }
    }

    /// Bumps this parser by one token. Returns the token that was previously the "current" token.
    fn bump(&mut self) -> LocatedToken {
        self.previous_token_location = self.current_token_location;
        let (next_next_token, next_next_token_comments) = self.read_token_internal();
        let next_token = std::mem::replace(&mut self.next_token, next_next_token);
        let token = std::mem::replace(&mut self.token, next_token);

        let next_comments =
            std::mem::replace(&mut self.next_token_comments, next_next_token_comments);
        let _ = std::mem::replace(&mut self.current_token_comments, next_comments);

        self.current_token_location = self.token.location();
        token
    }

    fn read_two_first_tokens(&mut self) {
        let (token, comments) = self.read_token_internal();
        self.token = token;
        self.current_token_comments = comments;
        self.current_token_location = self.token.location();

        let (token, comments) = self.read_token_internal();
        self.next_token = token;
        self.next_token_comments = comments;
    }

    fn read_token_internal(&mut self) -> (LocatedToken, String) {
        let mut last_comments = String::new();

        loop {
            match self.tokens.next() {
                Some(Ok(token)) => match token.token() {
                    Token::LineComment(comment, None) | Token::BlockComment(comment, None) => {
                        if !last_comments.is_empty() {
                            last_comments.push('\n');
                        }
                        last_comments.push_str(comment);
                        continue;
                    }
                    _ => {
                        return (token, last_comments);
                    }
                },
                Some(Err(lexer_error)) => self.errors.push(lexer_error.into()),
                None => {
                    let end_span = Span::single_char(self.current_token_location.span.end());
                    let end_location = Location::new(end_span, self.current_token_location.file);
                    let end_token = LocatedToken::new(Token::EOF, end_location);
                    return (end_token, last_comments);
                }
            }
        }
    }

    fn eat_kind(&mut self, kind: TokenKind) -> Option<LocatedToken> {
        if self.token.kind() == kind { Some(self.bump()) } else { None }
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

    /// Eats an identifier. If it's `_`, pushes an error but still returns it as an identifier.
    fn eat_non_underscore_ident(&mut self) -> Option<Ident> {
        let ident = self.eat_ident()?;
        if ident.as_str() == "_" {
            self.push_error(ParserErrorReason::ExpectedIdentifierGotUnderscore, ident.location());
        }
        Some(ident)
    }

    fn eat_ident(&mut self) -> Option<Ident> {
        if let Some(token) = self.eat_kind(TokenKind::Ident) {
            match token.into_token() {
                Token::Ident(ident) => Some(Ident::new(ident, self.previous_token_location)),
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

    fn eat_int(&mut self) -> Option<(FieldElement, Option<IntegerTypeSuffix>)> {
        if matches!(self.token.token(), Token::Int(..)) {
            let token = self.bump();
            match token.into_token() {
                Token::Int(int, suffix) => Some((int, suffix)),
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

    fn eat_unquote_marker(&mut self) -> Option<ExprId> {
        if let Some(token) = self.eat_kind(TokenKind::UnquoteMarker) {
            match token.into_token() {
                Token::UnquoteMarker(expr_id) => return Some(expr_id),
                _ => {
                    unreachable!("Expected only `UnquoteMarker` to have `TokenKind::UnquoteMarker`")
                }
            }
        }

        None
    }

    fn eat_attribute_start(&mut self) -> Option<bool> {
        if let Token::AttributeStart { is_inner: false, is_tag } = self.token.token() {
            // We have parsed the attribute start token `#[`.
            // Disable the "skip whitespaces" flag only for tag attributes so that the next `self.bump()`
            // does not consume the whitespace following the upcoming token.
            if *is_tag {
                self.set_lexer_skip_whitespaces_flag(false);
            }
            let token = self.bump();
            self.set_lexer_skip_whitespaces_flag(true);
            match token.into_token() {
                Token::AttributeStart { is_tag, .. } => Some(is_tag),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    fn eat_inner_attribute_start(&mut self) -> Option<bool> {
        if let Token::AttributeStart { is_inner: true, is_tag } = self.token.token() {
            // We have parsed the inner attribute start token `#![`.
            // Disable the "skip whitespaces" flag only for tag attributes so that the next `self.bump()`
            // does not consume the whitespace following the upcoming token.
            if *is_tag {
                self.set_lexer_skip_whitespaces_flag(false);
            }
            let token = self.bump();
            self.set_lexer_skip_whitespaces_flag(true);
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

    fn eat_semicolon(&mut self) -> bool {
        self.eat(Token::Semicolon)
    }

    fn eat_semicolons(&mut self) -> bool {
        if self.eat_semicolon() {
            while self.eat_semicolon() {
                self.push_error(
                    ParserErrorReason::UnexpectedSemicolon,
                    self.previous_token_location,
                );
            }
            true
        } else {
            false
        }
    }

    fn eat_semicolon_or_error(&mut self) {
        if !self.eat_semicolons() {
            self.expected_token(Token::Semicolon);
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

    fn at_whitespace(&self) -> bool {
        matches!(self.token.token(), Token::Whitespace(_))
    }

    fn set_lexer_skip_whitespaces_flag(&mut self, flag: bool) {
        if let TokenStream::Lexer(lexer) = &mut self.tokens {
            lexer.set_skip_whitespaces_flag(flag);
        };
    }

    fn next_is(&self, token: Token) -> bool {
        self.next_token.token() == &token
    }

    fn at_eof(&self) -> bool {
        self.token.token() == &Token::EOF
    }

    /// Skips tokens until we reach a recovery point (`;`, `}`, or EOF).
    /// This is used to recover from fatal parsing errors like exceeding
    /// the maximum recursion depth.
    ///
    /// The method tracks brace nesting to properly skip nested blocks,
    /// ensuring we don't stop at a `}` that belongs to an inner block.
    pub(super) fn skip_to_recovery_point(&mut self) {
        let mut brace_depth = 0;

        loop {
            match self.token.token() {
                Token::EOF => break,
                Token::Semicolon if brace_depth == 0 => {
                    // Don't consume the semicolon - let the caller handle it
                    break;
                }
                Token::LeftBrace => {
                    brace_depth += 1;
                    self.bump();
                }
                Token::RightBrace => {
                    if brace_depth == 0 {
                        // Don't consume - this closes a block we didn't open
                        break;
                    }
                    brace_depth -= 1;
                    self.bump();
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    fn location_since(&self, start_location: Location) -> Location {
        // When taking the span between locations in different files, just keep the first one
        if self.current_token_location.file != start_location.file {
            return start_location;
        }

        let start_span = start_location.span;

        let span = if self.current_token_location.span == start_location.span {
            start_span
        } else {
            let end_span = self.previous_token_location.span;
            if start_span.start() <= end_span.end() {
                Span::from(start_span.start()..end_span.end())
            } else {
                // TODO: workaround for now
                start_span
            }
        };

        Location::new(span, start_location.file)
    }

    fn location_at_previous_token_end(&self) -> Location {
        let span_at_previous_token_end = Span::from(
            self.previous_token_location.span.end()..self.previous_token_location.span.end(),
        );
        Location::new(span_at_previous_token_end, self.previous_token_location.file)
    }

    fn unknown_ident_at_previous_token_end(&self) -> Ident {
        Ident::new("(unknown)".to_string(), self.location_at_previous_token_end())
    }

    fn expected_identifier(&mut self) {
        self.expected_label(ParsingRuleLabel::Identifier);
    }

    fn expected_string(&mut self) {
        self.expected_label(ParsingRuleLabel::String);
    }

    fn expected_token(&mut self, token: Token) {
        self.errors.push(ParserError::expected_token(
            token,
            self.token.token().clone(),
            self.current_token_location,
        ));
    }

    fn expected_one_of_tokens(&mut self, tokens: &[Token]) {
        self.errors.push(ParserError::expected_one_of_tokens(
            tokens,
            self.token.token().clone(),
            self.current_token_location,
        ));
    }

    fn expected_label(&mut self, label: ParsingRuleLabel) {
        self.errors.push(ParserError::expected_label(
            label,
            self.token.token().clone(),
            self.current_token_location,
        ));
    }

    fn expected_token_separating_items(
        &mut self,
        token: Token,
        items: &'static str,
        location: Location,
    ) {
        self.push_error(
            ParserErrorReason::ExpectedTokenSeparatingTwoItems { token, items },
            location,
        );
    }

    #[allow(unused)]
    fn expected_mut_after_ampersand(&mut self) {
        self.push_error(
            ParserErrorReason::ExpectedMutAfterAmpersand { found: self.token.token().clone() },
            self.current_token_location,
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
                modifiers.visibility_location,
            );
        }
    }

    fn unconstrained_not_followed_by_an_item(&mut self, modifiers: Modifiers) {
        if let Some(location) = modifiers.unconstrained {
            self.push_error(ParserErrorReason::UnconstrainedNotFollowedByAnItem, location);
        }
    }

    fn comptime_not_followed_by_an_item(&mut self, modifiers: Modifiers) {
        if let Some(location) = modifiers.comptime {
            self.push_error(ParserErrorReason::ComptimeNotFollowedByAnItem, location);
        }
    }

    fn comptime_mutable_and_unconstrained_not_applicable(&mut self, modifiers: Modifiers) {
        self.mutable_not_applicable(modifiers);
        self.comptime_not_applicable(modifiers);
        self.unconstrained_not_applicable(modifiers);
    }

    fn mutable_not_applicable(&mut self, modifiers: Modifiers) {
        if let Some(location) = modifiers.mutable {
            self.push_error(ParserErrorReason::MutableNotApplicable, location);
        }
    }

    fn comptime_not_applicable(&mut self, modifiers: Modifiers) {
        if let Some(location) = modifiers.comptime {
            self.push_error(ParserErrorReason::ComptimeNotApplicable, location);
        }
    }

    fn unconstrained_not_applicable(&mut self, modifiers: Modifiers) {
        if let Some(location) = modifiers.unconstrained {
            self.push_error(ParserErrorReason::UnconstrainedNotApplicable, location);
        }
    }

    fn push_error(&mut self, reason: ParserErrorReason, location: Location) {
        self.errors.push(ParserError::with_reason(reason, location));
    }
}

fn eof_located_token() -> LocatedToken {
    LocatedToken::new(Token::EOF, Location::dummy())
}
