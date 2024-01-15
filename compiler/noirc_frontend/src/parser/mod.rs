//! The parser is the second pass of the noir compiler.
//! The parser's job is to take the output of the lexer (a stream of tokens)
//! and parse it into a valid Abstract Syntax Tree (Ast). During this, the parser
//! validates the grammar of the program and returns parsing errors for any syntactically
//! invalid constructs (such as `fn fn fn`).
//!
//! This file is mostly helper functions and types for the parser. For the parser itself,
//! see parser.rs. The definition of the abstract syntax tree can be found in the `ast` folder.
mod errors;
mod labels;
#[allow(clippy::module_inception)]
mod parser;

use crate::token::{Keyword, Token};
use crate::{ast::ImportStatement, Expression, NoirStruct};
use crate::{
    Ident, LetStatement, NoirFunction, NoirTrait, NoirTraitImpl, NoirTypeAlias, Recoverable,
    StatementKind, TypeImpl, UseTree,
};

use chumsky::prelude::*;
use chumsky::primitive::Container;
pub use errors::ParserError;
pub use errors::ParserErrorReason;
use noirc_errors::Span;
pub use parser::parse_program;

#[derive(Debug, Clone)]
pub(crate) enum TopLevelStatement {
    Function(NoirFunction),
    Module(Ident),
    Import(UseTree),
    Struct(NoirStruct),
    Trait(NoirTrait),
    TraitImpl(NoirTraitImpl),
    Impl(TypeImpl),
    TypeAlias(NoirTypeAlias),
    SubModule(ParsedSubModule),
    Global(LetStatement),
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
                Err(ParserError::empty(Token::EOF, span))
            } else {
                Ok(Recoverable::error(span))
            }
        })
}

/// Recovery strategy for statements: If a statement fails to parse skip until the next ';' or fail
/// if we find a '}' first.
fn statement_recovery() -> impl NoirParser<StatementKind> {
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
        .ignore_then(one_of([Token::Semicolon]))
        .map(|_| TopLevelStatement::Error)
}

/// Force the given parser to succeed, logging any errors it had
fn force<'a, T: 'a>(parser: impl NoirParser<T> + 'a) -> impl NoirParser<Option<T>> + 'a {
    parser.map(Some).recover_via(empty().map(|_| None))
}

#[derive(Clone, Default)]
pub struct SortedModule {
    pub imports: Vec<ImportStatement>,
    pub functions: Vec<NoirFunction>,
    pub types: Vec<NoirStruct>,
    pub traits: Vec<NoirTrait>,
    pub trait_impls: Vec<NoirTraitImpl>,
    pub impls: Vec<TypeImpl>,
    pub type_aliases: Vec<NoirTypeAlias>,
    pub globals: Vec<LetStatement>,

    /// Module declarations like `mod foo;`
    pub module_decls: Vec<Ident>,

    /// Full submodules as in `mod foo { ... definitions ... }`
    pub submodules: Vec<SortedSubModule>,
}

impl std::fmt::Display for SortedModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for decl in &self.module_decls {
            writeln!(f, "mod {decl};")?;
        }

        for import in &self.imports {
            write!(f, "{import}")?;
        }

        for global_const in &self.globals {
            write!(f, "{global_const}")?;
        }

        for type_ in &self.types {
            write!(f, "{type_}")?;
        }

        for function in &self.functions {
            write!(f, "{function}")?;
        }

        for impl_ in &self.impls {
            write!(f, "{impl_}")?;
        }

        for type_alias in &self.type_aliases {
            write!(f, "{type_alias}")?;
        }

        for submodule in &self.submodules {
            write!(f, "{submodule}")?;
        }

        Ok(())
    }
}

/// A ParsedModule contains an entire Ast for one file.
#[derive(Clone, Debug, Default)]
pub struct ParsedModule {
    pub items: Vec<Item>,
}

impl ParsedModule {
    pub fn into_sorted(self) -> SortedModule {
        let mut module = SortedModule::default();

        for item in self.items {
            match item.kind {
                ItemKind::Import(import) => module.push_import(import),
                ItemKind::Function(func) => module.push_function(func),
                ItemKind::Struct(typ) => module.push_type(typ),
                ItemKind::Trait(noir_trait) => module.push_trait(noir_trait),
                ItemKind::TraitImpl(trait_impl) => module.push_trait_impl(trait_impl),
                ItemKind::Impl(r#impl) => module.push_impl(r#impl),
                ItemKind::TypeAlias(type_alias) => module.push_type_alias(type_alias),
                ItemKind::Global(global) => module.push_global(global),
                ItemKind::ModuleDecl(mod_name) => module.push_module_decl(mod_name),
                ItemKind::Submodules(submodule) => module.push_submodule(submodule.into_sorted()),
            }
        }

        module
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ItemKind {
    Import(UseTree),
    Function(NoirFunction),
    Struct(NoirStruct),
    Trait(NoirTrait),
    TraitImpl(NoirTraitImpl),
    Impl(TypeImpl),
    TypeAlias(NoirTypeAlias),
    Global(LetStatement),
    ModuleDecl(Ident),
    Submodules(ParsedSubModule),
}

/// A submodule defined via `mod name { contents }` in some larger file.
/// These submodules always share the same file as some larger ParsedModule
#[derive(Clone, Debug)]
pub struct ParsedSubModule {
    pub name: Ident,
    pub contents: ParsedModule,
    pub is_contract: bool,
}

impl ParsedSubModule {
    pub fn into_sorted(self) -> SortedSubModule {
        SortedSubModule {
            name: self.name,
            contents: self.contents.into_sorted(),
            is_contract: self.is_contract,
        }
    }
}

impl std::fmt::Display for SortedSubModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mod {} {{", self.name)?;

        for line in self.contents.to_string().lines() {
            write!(f, "\n    {line}")?;
        }

        write!(f, "\n}}")
    }
}

#[derive(Clone)]
pub struct SortedSubModule {
    pub name: Ident,
    pub contents: SortedModule,
    pub is_contract: bool,
}

impl SortedModule {
    fn push_function(&mut self, func: NoirFunction) {
        self.functions.push(func);
    }

    fn push_type(&mut self, typ: NoirStruct) {
        self.types.push(typ);
    }

    fn push_trait(&mut self, noir_trait: NoirTrait) {
        self.traits.push(noir_trait);
    }

    fn push_trait_impl(&mut self, trait_impl: NoirTraitImpl) {
        self.trait_impls.push(trait_impl);
    }

    fn push_impl(&mut self, r#impl: TypeImpl) {
        self.impls.push(r#impl);
    }

    fn push_type_alias(&mut self, type_alias: NoirTypeAlias) {
        self.type_aliases.push(type_alias);
    }

    fn push_import(&mut self, import_stmt: UseTree) {
        self.imports.extend(import_stmt.desugar(None));
    }

    fn push_module_decl(&mut self, mod_name: Ident) {
        self.module_decls.push(mod_name);
    }

    fn push_submodule(&mut self, submodule: SortedSubModule) {
        self.submodules.push(submodule);
    }

    fn push_global(&mut self, global: LetStatement) {
        self.globals.push(global);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub enum Precedence {
    Lowest,
    Or,
    And,
    Xor,
    LessGreater,
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
            Token::Pipe => Precedence::Or,
            Token::Ampersand => Precedence::And,
            Token::Caret => Precedence::Xor,
            Token::Less => Precedence::LessGreater,
            Token::LessEqual => Precedence::LessGreater,
            Token::Greater => Precedence::LessGreater,
            Token::GreaterEqual => Precedence::LessGreater,
            Token::ShiftLeft => Precedence::Shift,
            Token::ShiftRight => Precedence::Shift,
            Token::Plus => Precedence::Sum,
            Token::Minus => Precedence::Sum,
            Token::Slash => Precedence::Product,
            Token::Star => Precedence::Product,
            Token::Percent => Precedence::Product,
            _ => return None,
        };

        assert_ne!(precedence, Precedence::Highest, "expression_with_precedence in the parser currently relies on the highest precedence level being uninhabited");
        Some(precedence)
    }

    /// Return the next higher precedence. E.g. `Sum.next() == Product`
    fn next(self) -> Self {
        use Precedence::*;
        match self {
            Lowest => Or,
            Or => Xor,
            Xor => And,
            And => LessGreater,
            LessGreater => Shift,
            Shift => Sum,
            Sum => Product,
            Product => Highest,
            Highest => Highest,
        }
    }

    /// TypeExpressions only contain basic arithmetic operators and
    /// notably exclude `>` due to parsing conflicts with generic type brackets.
    fn next_type_precedence(self) -> Self {
        use Precedence::*;
        match self {
            Lowest => Sum,
            Sum => Product,
            Product => Highest,
            Highest => Highest,
            other => unreachable!("Unexpected precedence level in type expression: {:?}", other),
        }
    }

    /// The operators with the lowest precedence still useable in type expressions
    /// are '+' and '-' with precedence Sum.
    fn lowest_type_precedence() -> Self {
        Precedence::Sum
    }
}

impl std::fmt::Display for TopLevelStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopLevelStatement::Function(fun) => fun.fmt(f),
            TopLevelStatement::Module(m) => write!(f, "mod {m}"),
            TopLevelStatement::Import(tree) => write!(f, "use {tree}"),
            TopLevelStatement::Trait(t) => t.fmt(f),
            TopLevelStatement::TraitImpl(i) => i.fmt(f),
            TopLevelStatement::Struct(s) => s.fmt(f),
            TopLevelStatement::Impl(i) => i.fmt(f),
            TopLevelStatement::TypeAlias(t) => t.fmt(f),
            TopLevelStatement::SubModule(s) => s.fmt(f),
            TopLevelStatement::Global(c) => c.fmt(f),
            TopLevelStatement::Error => write!(f, "error"),
        }
    }
}

impl std::fmt::Display for ParsedModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.clone().into_sorted().fmt(f)
    }
}

impl std::fmt::Display for ParsedSubModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.clone().into_sorted().fmt(f)
    }
}
