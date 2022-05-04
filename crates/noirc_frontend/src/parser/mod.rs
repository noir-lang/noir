mod errors;
#[allow(clippy::module_inception)]
mod parser;

use crate::token::{Keyword, Token};
use crate::{ast::ImportStatement, Expression, NoirStruct};
use crate::{Ident, NoirFunction, NoirImpl, Recoverable, Statement};

use chumsky::prelude::*;
use chumsky::primitive::Container;
pub use errors::ParserError;
use noirc_errors::Span;
pub use parser::parse_program;

#[derive(Debug, Clone)]
pub(crate) enum TopLevelStatement {
    Function(NoirFunction),
    Module(Ident),
    Import(ImportStatement),
    Struct(NoirStruct),
    Impl(NoirImpl),
    Error,
}

// Helper trait that gives us simpler type signatures for return types:
// e.g. impl Parser<T> versus impl Parser<Token, T, Error = Simple<Token>>
pub trait NoirParser<T>: Parser<Token, T, Error = ParserError> + Sized + Clone {}
impl<P, T> NoirParser<T> for P where P: Parser<Token, T, Error = ParserError> + Clone {}

// ExprParser just serves as a type alias for NoirParser<Expression> + Clone
trait ExprParser: NoirParser<Expression> {}
impl<P> ExprParser for P where P: NoirParser<Expression> {}

fn parenthesized<P, T>(parser: P) -> impl NoirParser<T>
where
    P: NoirParser<T>,
    T: Recoverable,
{
    use Token::*;
    parser.delimited_by(just(LeftParen), just(RightParen)).recover_with(nested_delimiters(
        LeftParen,
        RightParen,
        [(LeftBracket, RightBracket)],
        Recoverable::error,
    ))
}

fn spanned<P, T>(parser: P) -> impl NoirParser<(T, Span)>
where
    P: NoirParser<T>,
{
    parser.map_with_span(|value, span| (value, span))
}

// Parse with the first parser, then continue by
// repeating the second parser 0 or more times.
// The passed in function is then used to combine the
// results of both parsers along with their spans at
// each step.
fn foldl_with_span<P1, P2, T1, T2, F>(
    first_parser: P1,
    to_be_repeated: P2,
    f: F,
) -> impl NoirParser<T1>
where
    P1: NoirParser<T1>,
    P2: NoirParser<T2>,
    F: Fn(T1, T2, Span) -> T1 + Clone,
{
    spanned(first_parser)
        .then(spanned(to_be_repeated).repeated())
        .foldl(move |(a, a_span), (b, b_span)| {
            let span = a_span.merge(b_span);
            (f(a, b, span), span)
        })
        .map(|(value, _span)| value)
}

/// Sequence the two parsers.
/// Fails if the first parser fails, otherwise forces
/// the second parser to succeed while logging any errors.
fn then_commit<'a, P1, P2, T1, T2: 'a>(
    first_parser: P1,
    second_parser: P2,
) -> impl NoirParser<(T1, T2)> + 'a
where
    P1: NoirParser<T1> + 'a,
    P2: NoirParser<T2> + 'a,
    T2: Clone + Recoverable,
{
    let second_parser = skip_then_retry_until(second_parser)
        .map_with_span(|option, span| option.unwrap_or_else(|| Recoverable::error(span)));

    first_parser.then(second_parser)
}

fn then_commit_ignore<'a, P1, P2, T1: 'a, T2: 'a>(
    first_parser: P1,
    second_parser: P2,
) -> impl NoirParser<T1> + 'a
where
    P1: NoirParser<T1> + 'a,
    P2: NoirParser<T2> + 'a,
    T2: Clone,
{
    let second_parser = skip_then_retry_until(second_parser);
    first_parser.then_ignore(second_parser)
}

fn ignore_then_commit<'a, P1, P2, T1: 'a, T2: Clone + 'a>(
    first_parser: P1,
    second_parser: P2,
) -> impl NoirParser<T2> + 'a
where
    P1: NoirParser<T1> + 'a,
    P2: NoirParser<T2> + 'a,
    T2: Recoverable,
{
    let second_parser = skip_then_retry_until(second_parser)
        .map_with_span(|option, span| option.unwrap_or_else(|| Recoverable::error(span)));

    first_parser.ignore_then(second_parser)
}

fn skip_then_retry_until<'a, P, T: 'a>(parser: P) -> impl NoirParser<Option<T>> + 'a
where
    P: NoirParser<T> + 'a,
    T: Clone,
{
    let terminators = [
        Token::EOF,
        Token::Colon,
        Token::Semicolon,
        Token::RightBrace,
        Token::Keyword(Keyword::Let),
        Token::Keyword(Keyword::Constrain),
    ];
    force(parser.recover_with(chumsky::prelude::skip_then_retry_until(terminators)))
}

/// General recovery strategy: try to skip to the target token, failing if we encounter the
/// 'too_far' token beforehand.
///
/// Expects all of `too_far` to be contained within `targets`
fn try_skip_until<T, C1, C2>(targets: C1, too_far: C2) -> impl NoirParser<T>
where
    T: Recoverable + Clone,
    C1: Container<Token> + Clone,
    C2: Container<Token> + Clone,
{
    chumsky::prelude::none_of(targets)
        .repeated()
        .ignore_then(one_of(too_far.clone()).rewind())
        .try_map(move |peek, span| {
            if too_far.get_iter().any(|t| t == peek) {
                // This error will never be shown to the user
                Err(ParserError::with_reason(String::new(), span))
            } else {
                Ok(Recoverable::error(span))
            }
        })
}

/// Recovery strategy for statements: If a statement fails to parse skip until the next ';' or fail
/// if we find a '}' first.
fn statement_recovery() -> impl NoirParser<Statement> {
    use Token::*;
    try_skip_until([Semicolon, RightBrace], RightBrace)
}

fn parameter_recovery<T: Recoverable + Clone>() -> impl NoirParser<T> {
    use Token::*;
    try_skip_until([Comma, RightParen], RightParen)
}

fn parameter_name_recovery<T: Recoverable + Clone>() -> impl NoirParser<T> {
    use Token::*;
    try_skip_until([Colon, RightParen, Comma], [RightParen, Comma])
}

fn top_level_statement_recovery() -> impl NoirParser<TopLevelStatement> {
    none_of([Token::Semicolon, Token::RightBrace, Token::EOF])
        .repeated()
        .ignore_then(one_of([Token::Semicolon, Token::RightBrace]))
        .map(|_| TopLevelStatement::Error)
}

/// Force the given parser to succeed, logging any errors it had
fn force<'a, T: 'a>(parser: impl NoirParser<T> + 'a) -> impl NoirParser<Option<T>> + 'a {
    parser.map(Some).recover_via(empty().map(|_| None))
}

#[derive(Clone, Debug, Default)]
pub struct ParsedModule {
    pub imports: Vec<ImportStatement>,
    pub functions: Vec<NoirFunction>,
    pub types: Vec<NoirStruct>,
    pub impls: Vec<NoirImpl>,
    pub module_decls: Vec<Ident>,
}

impl ParsedModule {
    fn push_function(&mut self, func: NoirFunction) {
        self.functions.push(func);
    }

    fn push_type(&mut self, typ: NoirStruct) {
        self.types.push(typ);
    }

    fn push_impl(&mut self, r#impl: NoirImpl) {
        self.impls.push(r#impl);
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
    Or,
    Xor,
    And,
    Shift,
    Sum,
    Product,
    Highest,
}

impl Precedence {
    // Higher the number, the higher(more priority) the precedence
    // XXX: Check the precedence is correct for operators
    fn token_precedence(tok: &Token) -> Option<Precedence> {
        let precedence = match tok {
            Token::Equal => Precedence::Lowest,
            Token::NotEqual => Precedence::Lowest,
            Token::Less => Precedence::LessGreater,
            Token::LessEqual => Precedence::LessGreater,
            Token::Greater => Precedence::LessGreater,
            Token::GreaterEqual => Precedence::LessGreater,
            Token::Ampersand => Precedence::And,
            Token::Caret => Precedence::Xor,
            Token::Pipe => Precedence::Or,
            Token::Plus => Precedence::Sum,
            Token::Minus => Precedence::Sum,
            Token::ShiftLeft => Precedence::Shift,
            Token::ShiftRight => Precedence::Shift,
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
            LessGreater => Or,
            Or => Xor,
            Xor => And,
            And => Shift,
            Shift => Sum,
            Sum => Product,
            Product => Highest,
            Highest => Highest,
        }
    }
}

impl std::fmt::Display for TopLevelStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopLevelStatement::Function(fun) => fun.fmt(f),
            TopLevelStatement::Module(m) => write!(f, "mod {}", m),
            TopLevelStatement::Import(i) => i.fmt(f),
            TopLevelStatement::Struct(s) => s.fmt(f),
            TopLevelStatement::Impl(i) => i.fmt(f),
            TopLevelStatement::Error => write!(f, "error"),
        }
    }
}
