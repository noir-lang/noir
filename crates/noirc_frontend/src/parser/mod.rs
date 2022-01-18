mod errors;
#[allow(clippy::module_inception)]
mod parser;
mod combinators;

use std::ops::Range;

use crate::{ast::ImportStatement, NoirFunction};
use crate::token::Token;
use crate::Ident;

pub use errors::ParserErrorKind;
use noirc_errors::Span;
pub use parser::parse_program;
use chumsky::prelude::*;

pub type ParseError = Simple<SpannedToken>;
pub type ParserExprKindResult = Result<crate::ExpressionKind, ParseError>;
pub type ParserExprResult = Result<crate::Expression, ParseError>;

enum TopLevelStatement {
    Function(NoirFunction),
    Module(Ident),
    Import(ImportStatement),
}

use crate::token::SpannedToken;

// Helper trait that compared to using just Parser gives us:
// 1. Simpler type signatures (impl Parser<O> versus impl Parser<SpannedToken, O, Error = Simple<SpannedToken>>)
// 2. The ability to use member access syntax for some helper functions defined in the trait.
pub trait NoirParser<O>: Parser<SpannedToken, O, Error = ParseError> + Sized {
    fn surrounded_by(self, left: Token, right: Token) -> chumsky::combinator::DelimitedBy<Self, SpannedToken>;
    fn parenthesized(self) -> chumsky::combinator::DelimitedBy<Self, SpannedToken>;
    fn spanned(self) -> chumsky::combinator::MapWithSpan<Self, fn(O, Range<usize>) -> (O, Span), O>;
}

impl<P, O> NoirParser<O> for P where P: Parser<SpannedToken, O, Error = ParseError> {
    fn parenthesized(self) -> chumsky::combinator::DelimitedBy<Self, SpannedToken> {
        self.surrounded_by(Token::LeftParen, Token::RightParen)
    }

    fn surrounded_by(self, left: Token, right: Token) -> chumsky::combinator::DelimitedBy<Self, SpannedToken> {
        let left = SpannedToken::dummy_span(left);
        let right = SpannedToken::dummy_span(right);
        self.delimited_by(left, right)
    }

    fn spanned(self) -> chumsky::combinator::MapWithSpan<Self, fn(O, Range<usize>) -> (O, Span), O> {
        self.map_with_span(attach_span)
    }
}

fn foldl_with_span<P1, P2, O1, O2, F>(first_parser: P1, to_be_repeated: P2, f: F) -> impl NoirParser<O1>
    where P1: NoirParser<O1>,
          P2: NoirParser<O2>,
          F: Fn((O1, Span), (O2, Span)) -> O1
{
    first_parser.spanned()
        .then(to_be_repeated.spanned().repeated())
        .foldl(move |a, b| {
            let span = a.1.clone();
            (f(a, b), span)
        })
        .map(|(value, _span)| value)
}

fn attach_span<T>(value: T, range: Range<usize>) -> (T, Span) {
    (value, Span::new(range))
}


#[derive(Clone, Debug)]
pub struct ParsedModule {
    pub imports: Vec<ImportStatement>,
    pub functions: Vec<NoirFunction>,
    pub module_decls: Vec<Ident>,
}

impl ParsedModule {
    fn with_capacity(cap: usize) -> Self {
        ParsedModule {
            imports: Vec::with_capacity(cap),
            functions: Vec::with_capacity(cap),
            module_decls: Vec::new(),
        }
    }

    fn push_function(&mut self, func: NoirFunction) {
        self.functions.push(func);
    }
    fn push_import(&mut self, import_stmt: ImportStatement) {
        self.imports.push(import_stmt);
    }
    fn push_module_decl(&mut self, mod_name: Ident) {
        self.module_decls.push(mod_name);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Precedence {
    Lowest,
    LessGreater,
    Sum,
    Product,
    Highest,
}

impl Precedence {
    // Higher the number, the higher(more priority) the precedence
    // XXX: Check the precedence is correct for operators
    fn token_precedence(tok: &Token) -> Option<Precedence> {
        let precedence = match tok {
            Token::Assign => Precedence::Lowest,
            Token::Equal => Precedence::Lowest,
            Token::NotEqual => Precedence::Lowest,
            Token::Less => Precedence::LessGreater,
            Token::LessEqual => Precedence::LessGreater,
            Token::Greater => Precedence::LessGreater,
            Token::GreaterEqual => Precedence::LessGreater,
            Token::Ampersand => Precedence::Sum,
            Token::Caret => Precedence::Sum,
            Token::Pipe => Precedence::Sum,
            Token::Plus => Precedence::Sum,
            Token::Minus => Precedence::Sum,
            Token::Slash => Precedence::Product,
            Token::Star => Precedence::Product,
            _ => return None,
        };

        assert_ne!(precedence, Precedence::Highest, "expression_with_precedence in the parser currently relies on the highest precedence level being uninhabited");
        Some(precedence)
    }

    fn higher(self) -> Self {
        use Precedence::*;
        match self {
            Lowest => LessGreater,
            LessGreater => Sum,
            Sum => Product,
            Product => Highest,
            Highest => Highest,
        }
    }
}
