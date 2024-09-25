use noirc_errors::Span;

use crate::{
    ast::{
        Expression, Ident, ItemVisibility, LValue, Path, Pattern, Statement, TraitBound,
        UnresolvedType,
    },
    lexer::{Lexer, SpannedTokenResult},
    token::{Keyword, SpannedToken, Token, TokenKind, Tokens},
};

use super::{Item, ItemKind, ParsedModule, ParserError};

mod attributes;
mod doc_comments;
mod use_tree;

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
            TokenStream::Tokens(tokens) => {
                if let Some(token) = tokens.0.pop() {
                    Some(Ok(token))
                } else {
                    None
                }
            }
        }
    }
}

pub struct Parser<'a> {
    errors: Vec<ParserError>,
    tokens: TokenStream<'a>,
    token: SpannedToken,
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

    fn new(tokens: TokenStream<'a>) -> Self {
        let mut parser = Self {
            errors: Vec::new(),
            tokens,
            token: SpannedToken::new(Token::EOF, Default::default()),
            current_token_span: Default::default(),
            previous_token_span: Default::default(),
        };
        parser.next_token();
        parser
    }

    pub(crate) fn parse_module(&mut self) -> ParsedModule {
        let inner_doc_comments = self.parse_inner_doc_comments();
        let items = self.parse_items();

        ParsedModule { items, inner_doc_comments }
    }

    pub(crate) fn parse_items(&mut self) -> Vec<Item> {
        let mut items = Vec::new();

        while let Some(item) = self.parse_item() {
            items.push(item);
        }

        items
    }

    fn parse_item(&mut self) -> Option<Item> {
        let doc_comments = self.parse_outer_doc_comments();

        let start_span = self.current_token_span;
        let kind = self.parse_item_kind()?;
        let span = self.span_since(start_span);

        Some(Item { kind, span, doc_comments })
    }

    fn parse_item_kind(&mut self) -> Option<ItemKind> {
        if let Some(kind) = self.parse_inner_attribute() {
            return Some(kind);
        }

        let visibility = self.parse_item_visibility();

        if self.eat_keyword(Keyword::Use) {
            let use_tree = self.parse_use_tree();
            return Some(ItemKind::Import(use_tree, visibility));
        }

        None
    }

    fn parse_item_visibility(&mut self) -> ItemVisibility {
        if !self.eat_keyword(Keyword::Pub) {
            return ItemVisibility::Private;
        }

        if self.eat(Token::LeftParen).is_none() {
            // `pub`
            return ItemVisibility::Public;
        }

        if !self.eat_keyword(Keyword::Crate) {
            // TODO: error
            // `pub(` or `pub()`
            self.eat(Token::RightParen);
            return ItemVisibility::Public;
        }

        if self.eat(Token::RightParen).is_some() {
            // `pub(crate)`
            ItemVisibility::PublicCrate
        } else {
            // `pub(crate`
            // TODO: error
            ItemVisibility::PublicCrate
        }
    }

    pub(crate) fn parse_expression(&mut self) -> Expression {
        todo!("Parser")
    }

    pub(crate) fn parse_path_no_turbofish(&mut self) -> Path {
        todo!("Parser")
    }

    pub(crate) fn parse_pattern(&mut self) -> Pattern {
        todo!("Parser")
    }

    pub(crate) fn parse_type(&mut self) -> UnresolvedType {
        todo!("Parser")
    }

    pub(crate) fn parse_trait_bound(&mut self) -> TraitBound {
        todo!("Parser")
    }

    pub(crate) fn parse_statement(&mut self) -> Statement {
        todo!("Parser")
    }

    pub(crate) fn parse_lvalue(&mut self) -> LValue {
        todo!("Parser")
    }

    fn next_token(&mut self) {
        loop {
            self.previous_token_span = self.current_token_span;

            let token = self.tokens.next();
            if let Some(token) = token {
                match token {
                    Ok(token) => {
                        self.current_token_span = token.to_span();
                        self.token = token;
                        break;
                    }
                    Err(lexer_error) => self.errors.push(lexer_error.into()),
                }
            } else {
                self.token = SpannedToken::new(Token::EOF, Default::default());
                self.current_token_span = Default::default();
                break;
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

    fn eat_comma(&mut self) -> bool {
        self.eat(Token::Comma).is_some()
    }

    fn eat_commas(&mut self) -> bool {
        if self.eat_comma() {
            while self.eat_comma() {
                // TODO: error
            }
            true
        } else {
            false
        }
    }

    fn eat_semicolon(&mut self) -> bool {
        self.eat(Token::Semicolon).is_some()
    }

    fn eat_semicolons(&mut self) -> bool {
        if self.eat_semicolon() {
            while self.eat_semicolon() {
                // TODO: error
            }
            true
        } else {
            false
        }
    }

    fn eat(&mut self, token: Token) -> Option<()> {
        if self.token.token() == &token {
            // TODO: error
            self.next_token();
            Some(())
        } else {
            None
        }
    }

    fn span_since(&self, start_span: Span) -> Span {
        let end_span = self.previous_token_span;
        Span::from(start_span.start()..end_span.end())
    }
}
