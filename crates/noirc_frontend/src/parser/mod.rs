mod errors;
#[allow(clippy::module_inception)]
mod parser;

use crate::token::{Keyword, Token};
use crate::{ast::ImportStatement, Expression, NoirStruct};
use crate::{Ident, NoirFunction};

use chumsky::prelude::*;
pub use errors::ParserError;
use noirc_errors::Span;
pub use parser::parse_program;

#[derive(Debug)]
enum TopLevelStatement {
    Function(NoirFunction),
    Module(Ident),
    Import(ImportStatement),
    Struct(NoirStruct),
}

// Helper trait that gives us simpler type signatures for return types:
// e.g. impl Parser<T> versus impl Parser<Token, T, Error = Simple<Token>>
pub trait NoirParser<T>: Parser<Token, T, Error = ParserError> + Sized + Clone {}
impl<P, T> NoirParser<T> for P where P: Parser<Token, T, Error = ParserError> + Clone {}

// ExprParser just serves as a type alias for NoirParser<Expression> + Clone
trait ExprParser: NoirParser<Expression> {}
impl<P> ExprParser for P where P: NoirParser<Expression> {}

fn parenthesized<P, F, T>(parser: P, default: F) -> impl NoirParser<T>
where
    P: NoirParser<T>,
    F: Fn(Span) -> T + Clone,
{
    use Token::*;
    parser
        .delimited_by(just(LeftParen), just(RightParen))
        .recover_with(nested_delimiters(
            LeftParen,
            RightParen,
            [(LeftBracket, RightBracket)],
            default,
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
pub fn then_commit<'a, P1, P2, T1, T2: 'a, F>(
    first_parser: P1,
    second_parser: P2,
    default: F,
) -> impl NoirParser<(T1, T2)> + 'a
where
    P1: NoirParser<T1> + 'a,
    P2: NoirParser<T2> + 'a,
    F: 'a + Fn(Span) -> T2 + Clone,
    T2: Clone,
{
    let second_parser = skip_then_retry_until(second_parser)
        .map_with_span(move |option, span| option.unwrap_or_else(|| default(span)));

    first_parser.then(second_parser)
}

pub fn then_commit_ignore<'a, P1, P2, T1: 'a, T2: 'a>(
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

// pub fn foo<'a>() -> Custom<Box<dyn Fn(&'a mut Stream<SpannedToken, Span>) -> Result<(&'a [SpannedToken], i32), ParserError>>, ParserError> {
//     custom(Box::new(|stream| {
//         let tokens = stream.fetch_tokens();
//         todo!()
//     }))
// }

pub fn ignore_then_commit<'a, P1, P2, T1: 'a, T2: Clone + 'a, F>(
    first_parser: P1,
    second_parser: P2,
    default: F,
) -> impl NoirParser<T2> + 'a
where
    P1: NoirParser<T1> + 'a,
    P2: NoirParser<T2> + 'a,
    F: 'a + Fn(Span) -> T2 + Clone,
{
    let second_parser = skip_then_retry_until(second_parser)
        .map_with_span(move |option, span| option.unwrap_or_else(|| default(span)));

    first_parser.ignore_then(second_parser)
}

fn skip_then_retry_until<'a, P, T: 'a>(parser: P) -> impl NoirParser<Option<T>> + 'a
where
    P: NoirParser<T> + 'a,
    T: Clone,
{
    force(parser.clone()).then_with(move |option| match option {
        Some(value) => empty().map(move |_| Some(value.clone())).boxed(),
        None => {
            let parser = parser.clone();
            recursive(move |recur| {
                let terminators = [
                    Token::EOF,
                    Token::Colon,
                    Token::Semicolon,
                    Token::RightBrace,
                    Token::Keyword(Keyword::Let),
                    Token::Keyword(Keyword::Priv),
                    Token::Keyword(Keyword::Const),
                    Token::Keyword(Keyword::Constrain),
                ];

                parser.or(none_of(terminators).ignore_then(recur))
            })
            .or_not()
            .boxed()
        }
    })
}

/// Force the given parser to succeed, logging any errors it had
pub fn force<'a, P, T: 'a>(parser: P) -> impl NoirParser<Option<T>> + 'a
where
    P: NoirParser<T> + 'a,
{
    parser
        .clone()
        .map(Ok)
        .or_else(|err| Ok(Err(err)))
        .rewind()
        .then_with(move |result| match result {
            Ok(_) => parser.clone().map(Ok).boxed(),
            Err(err) => empty().map(move |_| Err(err.clone())).boxed(),
        })
        .validate(move |result, _, emit| match result {
            Ok(t) => Some(t),
            Err(err) => {
                emit(err);
                None
            }
        })
}

#[derive(Clone, Debug)]
pub struct ParsedModule {
    pub imports: Vec<ImportStatement>,
    pub functions: Vec<NoirFunction>,
    pub types: Vec<NoirStruct>,
    pub module_decls: Vec<Ident>,
}

impl ParsedModule {
    fn with_capacity(cap: usize) -> Self {
        ParsedModule {
            imports: Vec::with_capacity(cap),
            functions: Vec::with_capacity(cap),
            types: Vec::with_capacity(cap),
            module_decls: Vec::new(),
        }
    }

    fn push_function(&mut self, func: NoirFunction) {
        self.functions.push(func);
    }

    fn push_type(&mut self, typ: NoirStruct) {
        self.types.push(typ);
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

impl std::fmt::Display for TopLevelStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopLevelStatement::Function(fun) => fun.fmt(f),
            TopLevelStatement::Module(m) => write!(f, "mod {}", m),
            TopLevelStatement::Import(i) => i.fmt(f),
            TopLevelStatement::Struct(s) => s.fmt(f),
        }
    }
}
