//! This file contains the bulk of the implementation of noir's parser.
//!
//! Noir's parser is built off the [chumsky library](https://docs.rs/chumsky/latest/chumsky/)
//! for parser combinators. In this technique, parsers are built from smaller parsers that
//! parse e.g. only a single token. Then there are functions which can combine multiple
//! parsers together to create a larger one. These functions are called parser combinators.
//! For example, `a.then(b)` combines two parsers a and b and returns one that parses a
//! then parses b and fails if either fails. Other combinators like `a.or(b)` exist as well
//! and are used extensively. Note that these form a PEG grammar so if there are multiple
//! options as in `a.or(b)` the first matching parse will be chosen.
//!
//! Noir's grammar is not formally specified but can be estimated by inspecting each function.
//! For example, a function `f` parsing `choice((a, b, c))` can be roughly translated to
//! BNF as `f: a | b | c`.
//!
//! Occasionally there will also be recovery strategies present, either via `recover_via(Parser)`
//! or `recover_with(Strategy)`. The difference between the two functions isn't quite so important,
//! but both allow the parser to recover from a parsing error, log the error, and return an error
//! expression instead. These are used to parse cases such as `fn foo( { }` where we know the user
//! meant to write a function and thus we should log the error and return a function with no parameters,
//! rather than failing to parse a function and trying to subsequently parse a struct. Generally using
//! recovery strategies improves parser errors but using them incorrectly can be problematic since they
//! prevent other parsers from being tried afterward since there is no longer an error. Thus, they should
//! be limited to cases like the above `fn` example where it is clear we shouldn't back out of the
//! current parser to try alternative parsers in a `choice` expression.
use self::path::as_trait_path;
use self::primitives::{keyword, macro_quote_marker, mutable_reference, variable};
use self::types::{generic_type_args, maybe_comp_time};
pub use types::parse_type;

use super::{
    foldl_with_span, labels::ParsingRuleLabel, parameter_name_recovery, parameter_recovery,
    parenthesized, then_commit, then_commit_ignore, top_level_statement_recovery, ExprParser,
    NoirParser, ParsedModule, ParsedSubModule, ParserError, ParserErrorReason, Precedence,
    TopLevelStatement,
};
use super::{spanned, Item, ItemKind};
use crate::ast::{
    BinaryOp, BinaryOpKind, BlockExpression, ForLoopStatement, ForRange, Ident, IfExpression,
    InfixExpression, LValue, Literal, ModuleDeclaration, NoirTypeAlias, Param, Path, Pattern,
    Recoverable, Statement, TypeImpl, UnaryRhsMemberAccess, UnaryRhsMethodCall, UseTree,
    UseTreeKind, Visibility,
};
use crate::ast::{
    Expression, ExpressionKind, LetStatement, StatementKind, UnresolvedType, UnresolvedTypeData,
};
use crate::lexer::{lexer::from_spanned_token_result, Lexer};
use crate::parser::{force, ignore_then_commit, statement_recovery};
use crate::token::{Keyword, Token, TokenKind};
use acvm::AcirField;

use chumsky::prelude::*;
use iter_extended::vecmap;
use lalrpop_util::lalrpop_mod;
use noirc_errors::{Span, Spanned};

mod assertion;
mod attributes;
mod function;
mod lambdas;
mod literals;
pub(super) mod path;
mod primitives;
mod structs;
pub(super) mod traits;
mod types;

// synthesized by LALRPOP
lalrpop_mod!(pub noir_parser);

#[cfg(test)]
mod test_helpers;

use literals::literal;
use path::{maybe_empty_path, path};
use primitives::{dereference, ident, negation, not, nothing, right_shift_operator, token_kind};
use traits::where_clause;

/// Entry function for the parser - also handles lexing internally.
///
/// Given a source_program string, return the ParsedModule Ast representation
/// of the program along with any parsing errors encountered. If the parsing errors
/// Vec is non-empty, there may be Error nodes in the Ast to fill in the gaps that
/// failed to parse. Otherwise the Ast is guaranteed to have 0 Error nodes.
pub fn parse_program(source_program: &str) -> (ParsedModule, Vec<ParserError>) {
    let (tokens, lexing_errors) = Lexer::lex(source_program);
    let (module, mut parsing_errors) = program().parse_recovery_verbose(tokens);

    parsing_errors.extend(lexing_errors.into_iter().map(Into::into));
    let parsed_module = module.unwrap_or(ParsedModule { items: vec![] });

    if cfg!(feature = "experimental_parser") {
        for parsed_item in &parsed_module.items {
            if lalrpop_parser_supports_kind(&parsed_item.kind) {
                match &parsed_item.kind {
                    ItemKind::Import(parsed_use_tree) => {
                        prototype_parse_use_tree(Some(parsed_use_tree), source_program);
                    }
                    // other kinds prevented by lalrpop_parser_supports_kind
                    _ => unreachable!(),
                }
            }
        }
    }
    (parsed_module, parsing_errors)
}

fn prototype_parse_use_tree(expected_use_tree_opt: Option<&UseTree>, input: &str) {
    // TODO(https://github.com/noir-lang/noir/issues/4777): currently skipping
    // recursive use trees, e.g. "use std::{foo, bar}"
    if input.contains('{') {
        return;
    }

    let mut lexer = Lexer::new(input);
    lexer = lexer.skip_whitespaces(false);
    let mut errors = Vec::new();

    // NOTE: this is a hack to get the references working
    // => this likely means that we'll want to propagate the <'input> lifetime further into Token
    let lexer_result = lexer.collect::<Vec<_>>();
    let referenced_lexer_result = lexer_result.iter().map(from_spanned_token_result);

    let calculated = noir_parser::TopLevelStatementParser::new().parse(
        input,
        &mut errors,
        referenced_lexer_result,
    );

    if let Some(expected_use_tree) = expected_use_tree_opt {
        assert!(
            calculated.is_ok(),
            "calculated not Ok(_): {:?}\n\nlexer: {:?}\n\ninput: {:?}",
            calculated,
            lexer_result,
            input
        );

        match calculated.unwrap() {
            TopLevelStatement::Import(parsed_use_tree) => {
                assert_eq!(expected_use_tree, &parsed_use_tree);
            }
            unexpected_calculated => {
                panic!(
                    "expected a TopLevelStatement::Import, but found: {:?}",
                    unexpected_calculated
                )
            }
        }
    } else {
        assert!(
            calculated.is_err(),
            "calculated not Err(_): {:?}\n\nlexer: {:?}\n\ninput: {:?}",
            calculated,
            lexer_result,
            input
        );
    }
}

fn lalrpop_parser_supports_kind(kind: &ItemKind) -> bool {
    matches!(kind, ItemKind::Import(_))
}

/// program: module EOF
fn program() -> impl NoirParser<ParsedModule> {
    module().then_ignore(just(Token::EOF))
}

/// module: top_level_statement module
///       | %empty
fn module() -> impl NoirParser<ParsedModule> {
    recursive(|module_parser| {
        empty()
            .to(ParsedModule::default())
            .then(spanned(top_level_statement(module_parser)).repeated())
            .foldl(|mut program, (statement, span)| {
                if let Some(kind) = statement.into_item_kind() {
                    program.items.push(Item { kind, span });
                }
                program
            })
    })
}

/// This parser is used for parsing top level statements in macros
pub fn top_level_items() -> impl NoirParser<Vec<TopLevelStatement>> {
    top_level_statement(module()).repeated()
}

/// top_level_statement: function_definition
///                    | struct_definition
///                    | trait_definition
///                    | implementation
///                    | submodule
///                    | module_declaration
///                    | use_statement
///                    | global_declaration
fn top_level_statement<'a>(
    module_parser: impl NoirParser<ParsedModule> + 'a,
) -> impl NoirParser<TopLevelStatement> + 'a {
    choice((
        function::function_definition(false).map(TopLevelStatement::Function),
        structs::struct_definition(),
        traits::trait_definition(),
        traits::trait_implementation(),
        implementation(),
        type_alias_definition().then_ignore(force(just(Token::Semicolon))),
        submodule(module_parser.clone()),
        contract(module_parser),
        module_declaration().then_ignore(force(just(Token::Semicolon))),
        use_statement().then_ignore(force(just(Token::Semicolon))),
        global_declaration().then_ignore(force(just(Token::Semicolon))),
    ))
    .recover_via(top_level_statement_recovery())
}

/// Parses a non-trait implementation, adding a set of methods to a type.
///
/// implementation: 'impl' generics type '{' function_definition ... '}'
fn implementation() -> impl NoirParser<TopLevelStatement> {
    let methods_or_error = just(Token::LeftBrace)
        .ignore_then(spanned(function::function_definition(true)).repeated())
        .then_ignore(just(Token::RightBrace))
        .or_not()
        .validate(|methods, span, emit| {
            if let Some(methods) = methods {
                methods
            } else {
                emit(ParserError::with_reason(
                    ParserErrorReason::ExpectedLeftBracketOrWhereOrLeftBraceOrArrowAfterImplType,
                    span,
                ));
                vec![]
            }
        });

    keyword(Keyword::Impl)
        .ignore_then(function::generics())
        .then(parse_type().map_with_span(|typ, span| (typ, span)))
        .then(where_clause())
        .then(methods_or_error)
        .map(|args| {
            let ((other_args, where_clause), methods) = args;
            let (generics, (object_type, type_span)) = other_args;
            TopLevelStatement::Impl(TypeImpl {
                generics,
                object_type,
                type_span,
                where_clause,
                methods,
            })
        })
}

/// global_declaration: 'global' ident global_type_annotation '=' literal
fn global_declaration() -> impl NoirParser<TopLevelStatement> {
    let p = attributes::attributes()
        .then(maybe_comp_time())
        .then(spanned(keyword(Keyword::Mut)).or_not())
        .then_ignore(keyword(Keyword::Global).labelled(ParsingRuleLabel::Global))
        .then(ident().map(Pattern::Identifier));

    let p = then_commit(p, optional_type_annotation());
    let p = then_commit_ignore(p, just(Token::Assign));
    let p = then_commit(p, expression());
    p.validate(
        |(((((attributes, comptime), mutable), mut pattern), r#type), expression), span, emit| {
            let global_attributes =
                attributes::validate_secondary_attributes(attributes, span, emit);

            // Only comptime globals are allowed to be mutable, but we always parse the `mut`
            // and throw the error in name resolution.
            if let Some((_, mut_span)) = mutable {
                let span = mut_span.merge(pattern.span());
                pattern = Pattern::Mutable(Box::new(pattern), span, false);
            }
            LetStatement { pattern, r#type, comptime, expression, attributes: global_attributes }
        },
    )
    .map(TopLevelStatement::Global)
}

/// submodule: 'mod' ident '{' module '}'
fn submodule(module_parser: impl NoirParser<ParsedModule>) -> impl NoirParser<TopLevelStatement> {
    keyword(Keyword::Mod)
        .ignore_then(ident())
        .then_ignore(just(Token::LeftBrace))
        .then(module_parser)
        .then_ignore(just(Token::RightBrace))
        .map(|(name, contents)| {
            TopLevelStatement::SubModule(ParsedSubModule { name, contents, is_contract: false })
        })
}

/// contract: 'contract' ident '{' module '}'
fn contract(module_parser: impl NoirParser<ParsedModule>) -> impl NoirParser<TopLevelStatement> {
    keyword(Keyword::Contract)
        .ignore_then(ident())
        .then_ignore(just(Token::LeftBrace))
        .then(module_parser)
        .then_ignore(just(Token::RightBrace))
        .map(|(name, contents)| {
            TopLevelStatement::SubModule(ParsedSubModule { name, contents, is_contract: true })
        })
}

fn type_alias_definition() -> impl NoirParser<TopLevelStatement> {
    use self::Keyword::Type;

    let p = ignore_then_commit(keyword(Type), ident());
    let p = then_commit(p, function::generics());
    let p = then_commit_ignore(p, just(Token::Assign));
    let p = then_commit(p, parse_type());

    p.map_with_span(|((name, generics), typ), span| {
        TopLevelStatement::TypeAlias(NoirTypeAlias { name, generics, typ, span })
    })
}

fn self_parameter() -> impl NoirParser<Param> {
    let mut_ref_pattern = just(Token::Ampersand).then_ignore(keyword(Keyword::Mut));
    let mut_pattern = keyword(Keyword::Mut);

    mut_ref_pattern
        .or(mut_pattern)
        .map_with_span(|token, span| (token, span))
        .or_not()
        .then(filter_map(move |span, found: Token| match found {
            Token::Ident(ref word) if word == "self" => Ok(span),
            _ => Err(ParserError::expected_label(ParsingRuleLabel::Parameter, found, span)),
        }))
        .map(|(pattern_keyword, ident_span)| {
            let ident = Ident::new("self".to_string(), ident_span);
            let path = Path::from_single("Self".to_owned(), ident_span);
            let mut self_type = UnresolvedTypeData::Named(path, vec![], true).with_span(ident_span);
            let mut pattern = Pattern::Identifier(ident);

            match pattern_keyword {
                Some((Token::Ampersand, _)) => {
                    self_type = UnresolvedTypeData::MutableReference(Box::new(self_type))
                        .with_span(ident_span);
                }
                Some((Token::Keyword(_), span)) => {
                    pattern = Pattern::Mutable(Box::new(pattern), span.merge(ident_span), true);
                }
                _ => (),
            }

            Param { span: pattern.span(), pattern, typ: self_type, visibility: Visibility::Private }
        })
}

/// Function declaration parameters differ from other parameters in that parameter
/// patterns are not allowed in declarations. All parameters must be identifiers.
fn function_declaration_parameters() -> impl NoirParser<Vec<(Ident, UnresolvedType)>> {
    let typ = parse_type().recover_via(parameter_recovery());
    let typ = just(Token::Colon).ignore_then(typ);

    let full_parameter = ident().recover_via(parameter_name_recovery()).then(typ);
    let self_parameter = self_parameter().validate(|param, span, emit| {
        match param.pattern {
            Pattern::Identifier(ident) => (ident, param.typ),
            other => {
                emit(ParserError::with_reason(
                    ParserErrorReason::PatternInTraitFunctionParameter,
                    span,
                ));
                // into_ident panics on tuple or struct patterns but should be fine to call here
                // since the `self` parser can only parse `self`, `mut self` or `&mut self`.
                (other.into_ident(), param.typ)
            }
        }
    });

    let parameter = full_parameter.or(self_parameter);

    parameter
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .labelled(ParsingRuleLabel::Parameter)
}

fn block_expr<'a>(
    statement: impl NoirParser<StatementKind> + 'a,
) -> impl NoirParser<Expression> + 'a {
    block(statement).map(ExpressionKind::Block).map_with_span(Expression::new)
}

pub fn block<'a>(
    statement: impl NoirParser<StatementKind> + 'a,
) -> impl NoirParser<BlockExpression> + 'a {
    use Token::*;

    statement
        .recover_via(statement_recovery())
        .then(just(Semicolon).or_not().map_with_span(|s, span| (s, span)))
        .map_with_span(|(kind, rest), span| (Statement { kind, span }, rest))
        .repeated()
        .validate(check_statements_require_semicolon)
        .delimited_by(just(LeftBrace), just(RightBrace))
        .recover_with(nested_delimiters(
            LeftBrace,
            RightBrace,
            [(LeftParen, RightParen), (LeftBracket, RightBracket)],
            |span| vec![Statement { kind: StatementKind::Error, span }],
        ))
        .map(|statements| BlockExpression { statements })
}

fn check_statements_require_semicolon(
    statements: Vec<(Statement, (Option<Token>, Span))>,
    _span: Span,
    emit: &mut dyn FnMut(ParserError),
) -> Vec<Statement> {
    let last = statements.len().saturating_sub(1);
    let iter = statements.into_iter().enumerate();
    vecmap(iter, |(i, (statement, (semicolon, span)))| {
        statement.add_semicolon(semicolon, span, i == last, emit)
    })
}

/// Parse an optional ': type'
fn optional_type_annotation<'a>() -> impl NoirParser<UnresolvedType> + 'a {
    ignore_then_commit(just(Token::Colon), parse_type()).or_not().map_with_span(|r#type, span| {
        r#type.unwrap_or(UnresolvedTypeData::Unspecified.with_span(span))
    })
}

fn module_declaration() -> impl NoirParser<TopLevelStatement> {
    keyword(Keyword::Mod)
        .ignore_then(ident())
        .map(|ident| TopLevelStatement::Module(ModuleDeclaration { ident }))
}

fn use_statement() -> impl NoirParser<TopLevelStatement> {
    keyword(Keyword::Use).ignore_then(use_tree()).map(TopLevelStatement::Import)
}

fn rename() -> impl NoirParser<Option<Ident>> {
    ignore_then_commit(keyword(Keyword::As), ident()).or_not()
}

fn use_tree() -> impl NoirParser<UseTree> {
    recursive(|use_tree| {
        let simple = path::path_no_turbofish().then(rename()).map(|(mut prefix, alias)| {
            let ident = prefix.pop().ident;
            UseTree { prefix, kind: UseTreeKind::Path(ident, alias) }
        });

        let list = {
            let prefix = maybe_empty_path().then_ignore(just(Token::DoubleColon));
            let tree = use_tree
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(just(Token::LeftBrace), just(Token::RightBrace))
                .map(UseTreeKind::List);

            prefix.then(tree).map(|(prefix, kind)| UseTree { prefix, kind })
        };

        choice((list, simple))
    })
}

fn statement<'a, P, P2>(
    expr_parser: P,
    expr_no_constructors: P2,
) -> impl NoirParser<StatementKind> + 'a
where
    P: ExprParser + 'a,
    P2: ExprParser + 'a,
{
    recursive(|statement| {
        choice((
            assertion::constrain(expr_parser.clone()),
            assertion::assertion(expr_parser.clone()),
            assertion::assertion_eq(expr_parser.clone()),
            declaration(expr_parser.clone()),
            assignment(expr_parser.clone()),
            if_statement(expr_no_constructors.clone(), statement.clone()),
            block_statement(statement.clone()),
            for_loop(expr_no_constructors.clone(), statement.clone()),
            break_statement(),
            continue_statement(),
            return_statement(expr_parser.clone()),
            comptime_statement(expr_parser.clone(), expr_no_constructors, statement),
            expr_parser.map(StatementKind::Expression),
        ))
    })
}

pub fn fresh_statement() -> impl NoirParser<StatementKind> {
    statement(expression(), expression_no_constructors(expression()))
}

fn break_statement() -> impl NoirParser<StatementKind> {
    keyword(Keyword::Break).to(StatementKind::Break)
}

fn continue_statement() -> impl NoirParser<StatementKind> {
    keyword(Keyword::Continue).to(StatementKind::Continue)
}

fn comptime_statement<'a, P1, P2, S>(
    expr: P1,
    expr_no_constructors: P2,
    statement: S,
) -> impl NoirParser<StatementKind> + 'a
where
    P1: ExprParser + 'a,
    P2: ExprParser + 'a,
    S: NoirParser<StatementKind> + 'a,
{
    let comptime_statement = choice((
        declaration(expr),
        for_loop(expr_no_constructors, statement.clone()),
        block(statement).map_with_span(|block, span| {
            StatementKind::Expression(Expression::new(ExpressionKind::Block(block), span))
        }),
    ))
    .map_with_span(|kind, span| Box::new(Statement { kind, span }));

    keyword(Keyword::Comptime).ignore_then(comptime_statement).map(StatementKind::Comptime)
}

/// Comptime in an expression position only accepts entire blocks
fn comptime_expr<'a, S>(statement: S) -> impl NoirParser<ExpressionKind> + 'a
where
    S: NoirParser<StatementKind> + 'a,
{
    keyword(Keyword::Comptime)
        .ignore_then(spanned(block(statement)))
        .map(|(block, span)| ExpressionKind::Comptime(block, span))
}

fn unsafe_expr<'a, S>(statement: S) -> impl NoirParser<ExpressionKind> + 'a
where
    S: NoirParser<StatementKind> + 'a,
{
    keyword(Keyword::Unsafe)
        .ignore_then(spanned(block(statement)))
        .map(|(block, span)| ExpressionKind::Unsafe(block, span))
}

fn let_statement<'a, P>(
    expr_parser: P,
) -> impl NoirParser<((Pattern, UnresolvedType), Expression)> + 'a
where
    P: ExprParser + 'a,
{
    let p =
        ignore_then_commit(keyword(Keyword::Let).labelled(ParsingRuleLabel::Statement), pattern());
    let p = p.then(optional_type_annotation());
    let p = then_commit_ignore(p, just(Token::Assign));
    then_commit(p, expr_parser)
}

fn declaration<'a, P>(expr_parser: P) -> impl NoirParser<StatementKind> + 'a
where
    P: ExprParser + 'a,
{
    let_statement(expr_parser).map(StatementKind::new_let)
}

pub fn pattern() -> impl NoirParser<Pattern> {
    recursive(|pattern| {
        let ident_pattern = ident().map(Pattern::Identifier).map_err(|mut error| {
            if matches!(error.found(), Token::IntType(..)) {
                error = ParserError::with_reason(
                    ParserErrorReason::ExpectedPatternButFoundType(error.found().clone()),
                    error.span(),
                );
            }

            error
        });

        let mut_pattern = keyword(Keyword::Mut)
            .ignore_then(pattern.clone())
            .map_with_span(|inner, span| Pattern::Mutable(Box::new(inner), span, false));

        let short_field = ident().map(|name| (name.clone(), Pattern::Identifier(name)));
        let long_field = ident().then_ignore(just(Token::Colon)).then(pattern.clone());

        let struct_pattern_fields = long_field
            .or(short_field)
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::LeftBrace), just(Token::RightBrace));

        let struct_pattern = path(super::parse_type())
            .then(struct_pattern_fields)
            .map_with_span(|(typename, fields), span| Pattern::Struct(typename, fields, span));

        let tuple_pattern = pattern
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::LeftParen), just(Token::RightParen))
            .map_with_span(Pattern::Tuple);

        choice((mut_pattern, tuple_pattern, struct_pattern, ident_pattern))
    })
    .labelled(ParsingRuleLabel::Pattern)
}

fn assignment<'a, P>(expr_parser: P) -> impl NoirParser<StatementKind> + 'a
where
    P: ExprParser + 'a,
{
    let fallible =
        lvalue(expr_parser.clone()).then(assign_operator()).labelled(ParsingRuleLabel::Statement);

    then_commit(fallible, expr_parser).map_with_span(
        |((identifier, operator), expression), span| {
            StatementKind::assign(identifier, operator, expression, span)
        },
    )
}

/// Parse an assignment operator `=` optionally prefixed by a binary operator for a combined
/// assign statement shorthand. Notably, this must handle a few corner cases with how `>>` is
/// lexed as two separate greater-than operators rather than a single right-shift.
fn assign_operator() -> impl NoirParser<Token> {
    let shorthand_operators = Token::assign_shorthand_operators();
    // We need to explicitly check for right_shift here since it is actually
    // two separate greater-than operators.
    let shorthand_operators = right_shift_operator().or(one_of(shorthand_operators));
    let shorthand_syntax = shorthand_operators.then_ignore(just(Token::Assign));

    // Since >> is lexed as two separate "greater-than"s, >>= is lexed as > >=, so
    // we need to account for that case here as well.
    let right_shift_fix =
        just(Token::Greater).then(just(Token::GreaterEqual)).to(Token::ShiftRight);

    let shorthand_syntax = shorthand_syntax.or(right_shift_fix);
    just(Token::Assign).or(shorthand_syntax)
}

enum LValueRhs {
    MemberAccess(Ident, Span),
    Index(Expression, Span),
}

fn lvalue<'a, P>(expr_parser: P) -> impl NoirParser<LValue> + 'a
where
    P: ExprParser + 'a,
{
    recursive(|lvalue| {
        let l_ident = ident().map(LValue::Ident);

        let dereferences = just(Token::Star)
            .ignore_then(lvalue.clone())
            .map_with_span(|lvalue, span| LValue::Dereference(Box::new(lvalue), span));

        let parenthesized = lvalue.delimited_by(just(Token::LeftParen), just(Token::RightParen));

        let term = choice((parenthesized, dereferences, l_ident));

        let l_member_rhs =
            just(Token::Dot).ignore_then(field_name()).map_with_span(LValueRhs::MemberAccess);

        let l_index = expr_parser
            .delimited_by(just(Token::LeftBracket), just(Token::RightBracket))
            .map_with_span(LValueRhs::Index);

        term.then(l_member_rhs.or(l_index).repeated()).foldl(|lvalue, rhs| match rhs {
            LValueRhs::MemberAccess(field_name, span) => {
                let span = lvalue.span().merge(span);
                LValue::MemberAccess { object: Box::new(lvalue), field_name, span }
            }
            LValueRhs::Index(index, span) => {
                let span = lvalue.span().merge(span);
                LValue::Index { array: Box::new(lvalue), index, span }
            }
        })
    })
}

fn call_data() -> impl NoirParser<Visibility> {
    keyword(Keyword::CallData).then(parenthesized(literal())).validate(|token, span, emit| {
        match token {
            (_, ExpressionKind::Literal(Literal::Integer(x, _))) => {
                let id = x.to_u128() as u32;
                Visibility::CallData(id)
            }
            _ => {
                emit(ParserError::with_reason(ParserErrorReason::InvalidCallDataIdentifier, span));
                Visibility::CallData(0)
            }
        }
    })
}

fn optional_visibility() -> impl NoirParser<Visibility> {
    keyword(Keyword::Pub)
        .map(|_| Visibility::Public)
        .or(call_data())
        .or(keyword(Keyword::ReturnData).map(|_| Visibility::ReturnData))
        .or_not()
        .map(|opt| opt.unwrap_or(Visibility::Private))
}

pub fn expression() -> impl ExprParser {
    recursive(|expr| {
        expression_with_precedence(
            Precedence::Lowest,
            expr.clone(),
            expression_no_constructors(expr.clone()),
            statement(expr.clone(), expression_no_constructors(expr)),
            false,
            true,
        )
    })
    .labelled(ParsingRuleLabel::Expression)
}

fn expression_no_constructors<'a, P>(expr_parser: P) -> impl ExprParser + 'a
where
    P: ExprParser + 'a,
{
    recursive(|expr_no_constructors| {
        expression_with_precedence(
            Precedence::Lowest,
            expr_parser.clone(),
            expr_no_constructors.clone(),
            statement(expr_parser, expr_no_constructors),
            false,
            false,
        )
    })
    .labelled(ParsingRuleLabel::Expression)
}

fn return_statement<'a, P>(expr_parser: P) -> impl NoirParser<StatementKind> + 'a
where
    P: ExprParser + 'a,
{
    ignore_then_commit(keyword(Keyword::Return), expr_parser.or_not())
        .validate(|_, span, emit| {
            emit(ParserError::with_reason(ParserErrorReason::EarlyReturn, span));
            StatementKind::Error
        })
        .labelled(ParsingRuleLabel::Statement)
}

// An expression is a single term followed by 0 or more (OP subexpression)*
// where OP is an operator at the given precedence level and subexpression
// is an expression at the current precedence level plus one.
fn expression_with_precedence<'a, P, P2, S>(
    precedence: Precedence,
    expr_parser: P,
    expr_no_constructors: P2,
    statement: S,
    // True if we should only parse the restricted subset of operators valid within type expressions
    is_type_expression: bool,
    // True if we should also parse constructors `Foo { field1: value1, ... }` as an expression.
    // This is disabled when parsing the condition of an if statement due to a parsing conflict
    // with `then` bodies containing only a single variable.
    allow_constructors: bool,
) -> impl NoirParser<Expression> + 'a
where
    P: ExprParser + 'a,
    P2: ExprParser + 'a,
    S: NoirParser<StatementKind> + 'a,
{
    if precedence == Precedence::Highest {
        if is_type_expression {
            type_expression_term(expr_parser).boxed().labelled(ParsingRuleLabel::Term)
        } else {
            term(expr_parser, expr_no_constructors, statement, allow_constructors)
                .boxed()
                .labelled(ParsingRuleLabel::Term)
        }
    } else {
        let next_precedence =
            if is_type_expression { precedence.next_type_precedence() } else { precedence.next() };

        let next_expr = expression_with_precedence(
            next_precedence,
            expr_parser,
            expr_no_constructors,
            statement,
            is_type_expression,
            allow_constructors,
        );

        next_expr
            .clone()
            .then(then_commit(operator_with_precedence(precedence), next_expr).repeated())
            .foldl(create_infix_expression)
            .boxed()
            .labelled(ParsingRuleLabel::Expression)
    }
}

fn create_infix_expression(lhs: Expression, (operator, rhs): (BinaryOp, Expression)) -> Expression {
    let span = lhs.span.merge(rhs.span);
    let infix = Box::new(InfixExpression { lhs, operator, rhs });

    Expression { span, kind: ExpressionKind::Infix(infix) }
}

fn operator_with_precedence(precedence: Precedence) -> impl NoirParser<Spanned<BinaryOpKind>> {
    right_shift_operator()
        .or(any()) // Parse any single token, we're validating it as an operator next
        .try_map(move |token, span| {
            if Precedence::token_precedence(&token) == Some(precedence) {
                Ok(token.try_into_binary_op(span).unwrap())
            } else {
                Err(ParserError::expected_label(ParsingRuleLabel::BinaryOperator, token, span))
            }
        })
}

fn term<'a, P, P2, S>(
    expr_parser: P,
    expr_no_constructors: P2,
    statement: S,
    allow_constructors: bool,
) -> impl NoirParser<Expression> + 'a
where
    P: ExprParser + 'a,
    P2: ExprParser + 'a,
    S: NoirParser<StatementKind> + 'a,
{
    recursive(move |term_parser| {
        choice((
            not(term_parser.clone()),
            negation(term_parser.clone()),
            mutable_reference(term_parser.clone()),
            dereference(term_parser),
        ))
        .map_with_span(Expression::new)
        // right-unary operators like a[0] or a.f bind more tightly than left-unary
        // operators like  - or !, so that !a[0] is parsed as !(a[0]). This is a bit
        // awkward for casts so -a as i32 actually binds as -(a as i32).
        .or(atom_or_right_unary(
            expr_parser,
            expr_no_constructors,
            statement,
            allow_constructors,
            parse_type(),
        ))
    })
}

/// The equivalent of a 'term' for use in type expressions. Unlike regular terms, the grammar here
/// is restricted to no longer include right-unary expressions, unary not, and most atoms.
fn type_expression_term<'a, P>(expr_parser: P) -> impl NoirParser<Expression> + 'a
where
    P: ExprParser + 'a,
{
    recursive(move |term_parser| {
        negation(term_parser).map_with_span(Expression::new).or(type_expression_atom(expr_parser))
    })
}

fn atom_or_right_unary<'a, P, P2, S>(
    expr_parser: P,
    expr_no_constructors: P2,
    statement: S,
    allow_constructors: bool,
    type_parser: impl NoirParser<UnresolvedType> + 'a,
) -> impl NoirParser<Expression> + 'a
where
    P: ExprParser + 'a,
    P2: ExprParser + 'a,
    S: NoirParser<StatementKind> + 'a,
{
    enum UnaryRhs {
        Call((Option<Token>, Vec<Expression>)),
        ArrayIndex(Expression),
        Cast(UnresolvedType),
        MemberAccess(UnaryRhsMemberAccess),
        /// This is to allow `foo.` (no identifier afterwards) to be parsed as `foo`
        /// and produce an error, rather than just erroring (for LSP).
        JustADot,
    }

    // `(arg1, ..., argN)` in `my_func(arg1, ..., argN)`
    // Optionally accepts a leading `!` for macro calls.
    let call_rhs = just(Token::Bang)
        .or_not()
        .then(parenthesized(expression_list(expr_parser.clone())))
        .map(UnaryRhs::Call);

    // `[expr]` in `arr[expr]`
    let array_rhs = expr_parser
        .clone()
        .delimited_by(just(Token::LeftBracket), just(Token::RightBracket))
        .map(UnaryRhs::ArrayIndex);

    // `as Type` in `atom as Type`
    let cast_rhs = keyword(Keyword::As)
        .ignore_then(type_parser.clone())
        .map(UnaryRhs::Cast)
        .labelled(ParsingRuleLabel::Cast);

    // A turbofish operator is optional in a method call to specify generic types
    let turbofish = primitives::turbofish(type_parser);

    // `::<A, B>!(arg1, .., argN)` with the turbofish and macro portions being optional.
    let method_call_rhs = turbofish
        .then(just(Token::Bang).or_not())
        .then(parenthesized(expression_list(expr_parser.clone())))
        .map(|((turbofish, macro_call), args)| UnaryRhsMethodCall {
            turbofish,
            macro_call: macro_call.is_some(),
            args,
        });

    // `.foo` or `.foo(args)` in `atom.foo` or `atom.foo(args)`
    let member_rhs = just(Token::Dot)
        .ignore_then(field_name())
        .then(method_call_rhs.or_not())
        .map(|(method_or_field, method_call)| {
            UnaryRhs::MemberAccess(UnaryRhsMemberAccess { method_or_field, method_call })
        })
        .labelled(ParsingRuleLabel::FieldAccess);

    let just_a_dot =
        just(Token::Dot).map(|_| UnaryRhs::JustADot).validate(|value, span, emit_error| {
            emit_error(ParserError::with_reason(
                ParserErrorReason::ExpectedIdentifierAfterDot,
                span,
            ));
            value
        });

    let rhs = choice((call_rhs, array_rhs, cast_rhs, member_rhs, just_a_dot));

    foldl_with_span(
        atom(expr_parser, expr_no_constructors, statement, allow_constructors),
        rhs,
        |lhs, rhs, span| match rhs {
            UnaryRhs::Call((is_macro, args)) => {
                Expression::call(lhs, is_macro.is_some(), args, span)
            }
            UnaryRhs::ArrayIndex(index) => Expression::index(lhs, index, span),
            UnaryRhs::Cast(r#type) => Expression::cast(lhs, r#type, span),
            UnaryRhs::MemberAccess(field) => {
                Expression::member_access_or_method_call(lhs, field, span)
            }
            UnaryRhs::JustADot => lhs,
        },
    )
}

fn if_expr<'a, P, S>(expr_no_constructors: P, statement: S) -> impl NoirParser<ExpressionKind> + 'a
where
    P: ExprParser + 'a,
    S: NoirParser<StatementKind> + 'a,
{
    recursive(|if_parser| {
        let if_block = block_expr(statement.clone());
        // The else block could also be an `else if` block, in which case we must recursively parse it.
        let else_block = block_expr(statement).or(if_parser.map_with_span(|kind, span| {
            // Wrap the inner `if` expression in a block expression.
            // i.e. rewrite the sugared form `if cond1 {} else if cond2 {}` as `if cond1 {} else { if cond2 {} }`.
            let if_expression = Expression::new(kind, span);
            let desugared_else = BlockExpression {
                statements: vec![Statement {
                    kind: StatementKind::Expression(if_expression),
                    span,
                }],
            };
            Expression::new(ExpressionKind::Block(desugared_else), span)
        }));

        keyword(Keyword::If)
            .ignore_then(expr_no_constructors)
            .then(if_block.then(keyword(Keyword::Else).ignore_then(else_block).or_not()).or_not())
            .validate(|(condition, consequence_and_alternative), span, emit_error| {
                if let Some((consequence, alternative)) = consequence_and_alternative {
                    ExpressionKind::If(Box::new(IfExpression {
                        condition,
                        consequence,
                        alternative,
                    }))
                } else {
                    // We allow `if cond` without a block mainly for LSP, so that it parses right
                    // and autocompletion works there.
                    emit_error(ParserError::with_reason(
                        ParserErrorReason::ExpectedLeftBraceAfterIfCondition,
                        span,
                    ));

                    let span_end = condition.span.end();
                    ExpressionKind::If(Box::new(IfExpression {
                        condition,
                        consequence: Expression::new(
                            ExpressionKind::Error,
                            Span::from(span_end..span_end),
                        ),
                        alternative: None,
                    }))
                }
            })
    })
}

fn if_statement<'a, P, S>(
    expr_no_constructors: P,
    statement: S,
) -> impl NoirParser<StatementKind> + 'a
where
    P: ExprParser + 'a,
    S: NoirParser<StatementKind> + 'a,
{
    if_expr(expr_no_constructors, statement).map_with_span(|expression_kind, span| {
        StatementKind::Expression(Expression::new(expression_kind, span))
    })
}

fn block_statement<'a, S>(statement: S) -> impl NoirParser<StatementKind> + 'a
where
    S: NoirParser<StatementKind> + 'a,
{
    block(statement).map_with_span(|block, span| {
        StatementKind::Expression(Expression::new(ExpressionKind::Block(block), span))
    })
}

fn for_loop<'a, P, S>(expr_no_constructors: P, statement: S) -> impl NoirParser<StatementKind> + 'a
where
    P: ExprParser + 'a,
    S: NoirParser<StatementKind> + 'a,
{
    keyword(Keyword::For)
        .ignore_then(ident())
        .then_ignore(keyword(Keyword::In))
        .then(for_range(expr_no_constructors))
        .then(block_expr(statement))
        .map_with_span(|((identifier, range), block), span| {
            StatementKind::For(ForLoopStatement { identifier, range, block, span })
        })
}

/// The 'range' of a for loop. Either an actual range `start .. end` or an array expression.
fn for_range<P>(expr_no_constructors: P) -> impl NoirParser<ForRange>
where
    P: ExprParser,
{
    expr_no_constructors
        .clone()
        .then_ignore(just(Token::DoubleDot))
        .then(expr_no_constructors.clone())
        .map(|(start, end)| ForRange::Range(start, end))
        .or(expr_no_constructors.map(ForRange::Array))
}

fn array_expr<P>(expr_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    standard_array(expr_parser.clone()).or(array_sugar(expr_parser))
}

/// [a, b, c, ...]
fn standard_array<P>(expr_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    expression_list(expr_parser)
        .delimited_by(just(Token::LeftBracket), just(Token::RightBracket))
        .validate(|elements, _span, _emit| ExpressionKind::array(elements))
}

/// [a; N]
fn array_sugar<P>(expr_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    expr_parser
        .clone()
        .then(just(Token::Semicolon).ignore_then(expr_parser))
        .delimited_by(just(Token::LeftBracket), just(Token::RightBracket))
        .map(|(lhs, count)| ExpressionKind::repeated_array(lhs, count))
}

fn slice_expr<P>(expr_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    just(Token::Ampersand)
        .ignore_then(standard_slice(expr_parser.clone()).or(slice_sugar(expr_parser)))
}

/// &[a, b, c, ...]
fn standard_slice<P>(expr_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    expression_list(expr_parser)
        .delimited_by(just(Token::LeftBracket), just(Token::RightBracket))
        .validate(|elements, _span, _emit| ExpressionKind::slice(elements))
}

/// &[a; N]
fn slice_sugar<P>(expr_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    expr_parser
        .clone()
        .then(just(Token::Semicolon).ignore_then(expr_parser))
        .delimited_by(just(Token::LeftBracket), just(Token::RightBracket))
        .map(|(lhs, count)| ExpressionKind::repeated_slice(lhs, count))
}

fn expression_list<P>(expr_parser: P) -> impl NoirParser<Vec<Expression>>
where
    P: ExprParser,
{
    expr_parser.separated_by(just(Token::Comma)).allow_trailing()
}

/// Atoms are parameterized on whether constructor expressions are allowed or not.
/// Certain constructs like `if` and `for` disallow constructor expressions when a
/// block may be expected.
fn atom<'a, P, P2, S>(
    expr_parser: P,
    expr_no_constructors: P2,
    statement: S,
    allow_constructors: bool,
) -> impl NoirParser<Expression> + 'a
where
    P: ExprParser + 'a,
    P2: ExprParser + 'a,
    S: NoirParser<StatementKind> + 'a,
{
    choice((
        if_expr(expr_no_constructors, statement.clone()),
        slice_expr(expr_parser.clone()),
        array_expr(expr_parser.clone()),
        if allow_constructors {
            constructor(expr_parser.clone()).boxed()
        } else {
            nothing().boxed()
        },
        lambdas::lambda(expr_parser.clone()),
        block(statement.clone()).map(ExpressionKind::Block),
        comptime_expr(statement.clone()),
        unsafe_expr(statement),
        quote(),
        unquote(expr_parser.clone()),
        variable(),
        literal(),
        as_trait_path(parse_type()).map(ExpressionKind::AsTraitPath),
        macro_quote_marker(),
    ))
    .map_with_span(Expression::new)
    .or(parenthesized(expr_parser.clone()).map_with_span(|sub_expr, span| {
        Expression::new(ExpressionKind::Parenthesized(sub_expr.into()), span)
    }))
    .or(tuple(expr_parser))
    .labelled(ParsingRuleLabel::Atom)
}

/// Atoms within type expressions are limited to only variables, literals, and parenthesized
/// type expressions.
fn type_expression_atom<'a, P>(expr_parser: P) -> impl NoirParser<Expression> + 'a
where
    P: ExprParser + 'a,
{
    primitives::variable_no_turbofish()
        .or(literal())
        .map_with_span(Expression::new)
        .or(parenthesized(expr_parser))
        .labelled(ParsingRuleLabel::Atom)
}

fn quote() -> impl NoirParser<ExpressionKind> {
    token_kind(TokenKind::Quote).map(|token| {
        ExpressionKind::Quote(match token {
            Token::Quote(tokens) => tokens,
            _ => unreachable!("token_kind(Quote) should guarantee parsing only a quote token"),
        })
    })
}

/// unquote: '$' variable
///        | '$' '(' expression ')'
fn unquote<'a, P>(expr_parser: P) -> impl NoirParser<ExpressionKind> + 'a
where
    P: ExprParser + 'a,
{
    let unquote = variable().map_with_span(Expression::new).or(parenthesized(expr_parser));
    just(Token::DollarSign).ignore_then(unquote).map(|expr| ExpressionKind::Unquote(Box::new(expr)))
}

fn tuple<P>(expr_parser: P) -> impl NoirParser<Expression>
where
    P: ExprParser,
{
    parenthesized(expression_list(expr_parser)).map_with_span(|elements, span| {
        let kind = if elements.is_empty() {
            ExpressionKind::Literal(Literal::Unit)
        } else {
            ExpressionKind::Tuple(elements)
        };
        Expression::new(kind, span)
    })
}

fn field_name() -> impl NoirParser<Ident> {
    ident().or(token_kind(TokenKind::Literal).validate(|token, span, emit| match token {
        Token::Int(_) => Ident::from(Spanned::from(span, token.to_string())),
        other => {
            emit(ParserError::with_reason(ParserErrorReason::ExpectedFieldName(other), span));
            Ident::error(span)
        }
    }))
}

fn constructor(expr_parser: impl ExprParser) -> impl NoirParser<ExpressionKind> {
    let args = constructor_field(expr_parser)
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .delimited_by(just(Token::LeftBrace), just(Token::RightBrace));

    path(super::parse_type()).then(args).map(ExpressionKind::constructor)
}

fn constructor_field<P>(expr_parser: P) -> impl NoirParser<(Ident, Expression)>
where
    P: ExprParser,
{
    let long_form = ident().then_ignore(just(Token::Colon)).then(expr_parser);
    let short_form = ident().map(|ident| (ident.clone(), ident.into()));
    long_form.or(short_form)
}

#[cfg(test)]
mod test {
    use super::test_helpers::*;
    use super::*;
    use crate::ast::ArrayLiteral;

    #[test]
    fn parse_infix() {
        let valid = vec!["x + 6", "x - k", "x + (x + a)", " x * (x + a) + (x - 4)"];
        parse_all(expression(), valid);
        parse_all_failing(expression(), vec!["y ! x"]);
    }

    #[test]
    fn parse_function_call() {
        let valid = vec![
            "std::hash ()",
            " std::hash(x,y,a+b)",
            "crate::foo (x)",
            "hash (x,)",
            "(foo + bar)()",
            "(bar)()()()",
        ];
        parse_all(expression(), valid);
    }

    #[test]
    fn parse_cast() {
        let expression_nc = expression_no_constructors(expression());
        parse_all(
            atom_or_right_unary(
                expression(),
                expression_no_constructors(expression()),
                fresh_statement(),
                true,
                parse_type(),
            ),
            vec!["x as u8", "x as u16", "0 as Field", "(x + 3) as [Field; 8]"],
        );
        parse_all_failing(
            atom_or_right_unary(expression(), expression_nc, fresh_statement(), true, parse_type()),
            vec!["x as pub u8"],
        );
    }

    #[test]
    fn parse_array_index() {
        let valid = vec![
            "x[9]",
            "y[x+a]",
            " foo [foo+5]",
            "baz[bar]",
            "foo.bar[3] as Field .baz as u32 [7]",
        ];
        parse_all(
            atom_or_right_unary(
                expression(),
                expression_no_constructors(expression()),
                fresh_statement(),
                true,
                parse_type(),
            ),
            valid,
        );
    }

    fn expr_to_array(expr: ExpressionKind) -> ArrayLiteral {
        let lit = match expr {
            ExpressionKind::Literal(literal) => literal,
            _ => unreachable!("expected a literal"),
        };

        match lit {
            Literal::Array(arr) => arr,
            _ => unreachable!("expected an array"),
        }
    }

    #[test]
    fn parse_array() {
        let valid = vec![
            "[0, 1, 2,3, 4]",
            "[0,1,2,3,4,]", // Trailing commas are valid syntax
            "[0;5]",
        ];

        for expr in parse_all(array_expr(expression()), valid) {
            match expr_to_array(expr) {
                ArrayLiteral::Standard(elements) => assert_eq!(elements.len(), 5),
                ArrayLiteral::Repeated { length, .. } => {
                    assert_eq!(length.kind, ExpressionKind::integer(5i128.into()));
                }
            }
        }

        parse_all_failing(
            array_expr(expression()),
            vec!["0,1,2,3,4]", "[[0,1,2,3,4]", "[0,1,2,,]", "[0,1,2,3,4"],
        );
    }

    #[test]
    fn parse_array_sugar() {
        let valid = vec!["[0;7]", "[(1, 2); 4]", "[0;Four]", "[2;1+3-a]"];
        parse_all(array_expr(expression()), valid);

        let invalid = vec!["[0;;4]", "[1, 2; 3]"];
        parse_all_failing(array_expr(expression()), invalid);
    }

    fn expr_to_slice(expr: ExpressionKind) -> ArrayLiteral {
        let lit = match expr {
            ExpressionKind::Literal(literal) => literal,
            _ => unreachable!("expected a literal"),
        };

        match lit {
            Literal::Slice(arr) => arr,
            _ => unreachable!("expected a slice: {:?}", lit),
        }
    }

    #[test]
    fn parse_slice() {
        let valid = vec![
            "&[0, 1, 2,3, 4]",
            "&[0,1,2,3,4,]", // Trailing commas are valid syntax
            "&[0;5]",
        ];

        for expr in parse_all(slice_expr(expression()), valid) {
            match expr_to_slice(expr) {
                ArrayLiteral::Standard(elements) => assert_eq!(elements.len(), 5),
                ArrayLiteral::Repeated { length, .. } => {
                    assert_eq!(length.kind, ExpressionKind::integer(5i128.into()));
                }
            }
        }

        parse_all_failing(
            slice_expr(expression()),
            vec!["0,1,2,3,4]", "&[[0,1,2,3,4]", "&[0,1,2,,]", "&[0,1,2,3,4"],
        );
    }

    #[test]
    fn parse_slice_sugar() {
        let valid = vec!["&[0;7]", "&[(1, 2); 4]", "&[0;Four]", "&[2;1+3-a]"];
        parse_all(slice_expr(expression()), valid);

        let invalid = vec!["&[0;;4]", "&[1, 2; 3]"];
        parse_all_failing(slice_expr(expression()), invalid);
    }

    #[test]
    fn parse_block() {
        parse_with(block(fresh_statement()), "{ [0,1,2,3,4] }").unwrap();

        parse_all_failing(
            block(fresh_statement()),
            vec![
                "[0,1,2,3,4] }",
                "{ [0,1,2,3,4]",
                "{ [0,1,2,,] }", // Contents of the block must still be a valid expression
                "{ [0,1,2,3 }",
                "{ 0,1,2,3] }",
                "[[0,1,2,3,4]}",
            ],
        );
    }

    #[test]
    fn parse_let() {
        // Why is it valid to specify a let declaration as having type u8?
        //
        // Let statements are not type checked here, so the parser will accept as
        // long as it is a type. Other statements such as Public are type checked
        // Because for now, they can only have one type
        parse_all(
            declaration(expression()),
            vec!["let _ = 42", "let x = y", "let x : u8 = y", "let x: u16 = y"],
        );
    }

    #[test]
    fn parse_invalid_pub() {
        // pub cannot be used to declare a statement
        parse_all_failing(fresh_statement(), vec!["pub x = y", "pub x : pub Field = y"]);
    }

    #[test]
    fn parse_for_loop() {
        parse_all(
            for_loop(expression_no_constructors(expression()), fresh_statement()),
            vec!["for i in x+y..z {}", "for i in 0..100 { foo; bar }"],
        );

        parse_all_failing(
            for_loop(expression_no_constructors(expression()), fresh_statement()),
            vec![
                "for 1 in x+y..z {}",  // Cannot have a literal as the loop identifier
                "for i in 0...100 {}", // Only '..' is supported, there are no inclusive ranges yet
                "for i in 0..=100 {}", // Only '..' is supported, there are no inclusive ranges yet
            ],
        );
    }

    #[test]
    fn parse_parenthesized_expression() {
        parse_all(
            atom(expression(), expression_no_constructors(expression()), fresh_statement(), true),
            vec!["(0)", "(x+a)", "({(({{({(nested)})}}))})"],
        );
        parse_all_failing(
            atom(expression(), expression_no_constructors(expression()), fresh_statement(), true),
            vec!["(x+a", "((x+a)", "(,)"],
        );
    }

    #[test]
    fn parse_tuple() {
        parse_all(tuple(expression()), vec!["()", "(x,)", "(a,b+2)", "(a,(b,c,),d,)"]);
    }

    #[test]
    fn parse_if_expr() {
        parse_all(
            if_expr(expression_no_constructors(expression()), fresh_statement()),
            vec!["if x + a {  } else {  }", "if x {}", "if x {} else if y {} else {}"],
        );

        parse_all_failing(
            if_expr(expression_no_constructors(expression()), fresh_statement()),
            vec!["if (x / a) + 1 {} else", "if foo then 1 else 2", "if true { 1 }else 3"],
        );
    }

    #[test]
    fn parse_if_without_block() {
        let src = "if foo";
        let parser = if_expr(expression_no_constructors(expression()), fresh_statement());
        let (expression_kind, errors) = parse_recover(parser, src);

        let expression_kind = expression_kind.unwrap();
        let ExpressionKind::If(if_expression) = expression_kind else {
            panic!("Expected an if expression, got {:?}", expression_kind);
        };

        assert_eq!(if_expression.consequence.kind, ExpressionKind::Error);
        assert_eq!(if_expression.alternative, None);

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "expected { after if condition");
    }

    #[test]
    fn parse_module_declaration() {
        parse_with(module_declaration(), "mod foo").unwrap();
        parse_with(module_declaration(), "mod 1").unwrap_err();
    }

    #[test]
    fn parse_use() {
        let valid_use_statements = [
            "use std::hash",
            "use std",
            "use foo::bar as hello",
            "use bar as bar",
            "use foo::{}",
            "use foo::{bar,}",
            "use foo::{bar, hello}",
            "use foo::{bar as bar2, hello}",
            "use foo::{bar as bar2, hello::{foo}, nested::{foo, bar}}",
            "use std::{println, bar::baz}",
        ];

        let invalid_use_statements = [
            "use std as ;",
            "use foobar as as;",
            "use hello:: as foo;",
            "use foo bar::baz",
            "use foo bar::{baz}",
            "use foo::{,}",
        ];

        let use_statements = valid_use_statements
            .into_iter()
            .map(|valid_str| (valid_str, true))
            .chain(invalid_use_statements.into_iter().map(|invalid_str| (invalid_str, false)));

        for (use_statement_str, expect_valid) in use_statements {
            let mut use_statement_str = use_statement_str.to_string();
            let expected_use_statement = if expect_valid {
                let (result_opt, _diagnostics) =
                    parse_recover(&use_statement(), &use_statement_str);
                use_statement_str.push(';');
                match result_opt.unwrap() {
                    TopLevelStatement::Import(expected_use_statement) => {
                        Some(expected_use_statement)
                    }
                    _ => unreachable!(),
                }
            } else {
                let result = parse_with(&use_statement(), &use_statement_str);
                assert!(result.is_err());
                None
            };

            prototype_parse_use_tree(expected_use_statement.as_ref(), &use_statement_str);
        }
    }

    #[test]
    fn parse_type_aliases() {
        let cases = vec!["type foo = u8", "type bar = String", "type baz<T> = Vec<T>"];
        parse_all(type_alias_definition(), cases);

        let failing = vec!["type = u8", "type foo", "type foo = 1"];
        parse_all_failing(type_alias_definition(), failing);
    }

    #[test]
    fn parse_member_access() {
        let cases = vec!["a.b", "a + b.c", "foo.bar as u32"];
        parse_all(expression(), cases);
    }

    #[test]
    fn parse_constructor() {
        let cases = vec![
            "Baz",
            "Bar { ident: 32 }",
            "Baz { other: 2 + 42, ident: foo() + 1 }",
            "Baz { other, ident: foo() + 1, foo }",
        ];

        parse_all(expression(), cases);
        parse_with(expression(), "Foo { a + b }").unwrap_err();
    }

    // Semicolons are:
    // - Required after non-expression statements
    // - Optional after for, if, block expressions
    // - Optional after an expression as the last statement of a block
    // - Required after an expression as the non-final statement of a block
    #[test]
    fn parse_semicolons() {
        let cases = vec![
            "{ if true {} if false {} foo }",
            "{ if true {}; if false {} foo }",
            "{ for x in 0..1 {} if false {} foo; }",
            "{ let x = 2; }",
            "{ expr1; expr2 }",
            "{ expr1; expr2; }",
        ];
        parse_all(block(fresh_statement()), cases);

        let failing = vec![
            // We disallow multiple semicolons after a statement unlike rust where it is a warning
            "{ test;; foo }",
            "{ for x in 0..1 {} foo if false {} }",
            "{ let x = 2 }",
            "{ expr1 expr2 }",
        ];
        parse_all_failing(block(fresh_statement()), failing);
    }

    #[test]
    fn statement_recovery() {
        let cases = vec![
            Case { source: "let a = 4 + 3", expect: "let a: unspecified = (4 + 3)", errors: 0 },
            Case { source: "let a: = 4 + 3", expect: "let a: error = (4 + 3)", errors: 1 },
            Case { source: "let = 4 + 3", expect: "let $error: unspecified = (4 + 3)", errors: 1 },
            Case { source: "let = ", expect: "let $error: unspecified = Error", errors: 2 },
            Case { source: "let", expect: "let $error: unspecified = Error", errors: 3 },
            Case { source: "foo = one two three", expect: "foo = one", errors: 1 },
            Case { source: "constrain", expect: "constrain Error", errors: 2 },
            Case { source: "assert", expect: "constrain Error", errors: 1 },
            Case { source: "constrain x ==", expect: "constrain (x == Error)", errors: 2 },
            Case { source: "assert(x ==)", expect: "constrain (x == Error)", errors: 1 },
            Case { source: "assert(x == x, x)", expect: "constrain (x == x)", errors: 0 },
            Case { source: "assert_eq(x,)", expect: "constrain (Error == Error)", errors: 1 },
            Case {
                source: "assert_eq(x, x, x, x)",
                expect: "constrain (Error == Error)",
                errors: 1,
            },
            Case { source: "assert_eq(x, x, x)", expect: "constrain (x == x)", errors: 0 },
        ];

        check_cases_with_errors(&cases[..], fresh_statement());
    }

    #[test]
    fn return_validation() {
        let cases = [
            Case {
                source: "{ return 42; }",
                expect: concat!("{\n", "    Error\n", "}",),
                errors: 1,
            },
            Case {
                source: "{ return 1; return 2; }",
                expect: concat!("{\n", "    Error\n", "    Error\n", "}"),
                errors: 2,
            },
            Case {
                source: "{ return 123; let foo = 4 + 3; }",
                expect: concat!("{\n", "    Error\n", "    let foo: unspecified = (4 + 3)\n", "}"),
                errors: 1,
            },
            Case {
                source: "{ return 1 + 2 }",
                expect: concat!("{\n", "    Error\n", "}",),
                errors: 2,
            },
            Case { source: "{ return; }", expect: concat!("{\n", "    Error\n", "}",), errors: 1 },
        ];

        check_cases_with_errors(&cases[..], block(fresh_statement()));
    }

    #[test]
    fn expr_no_constructors() {
        let cases = [
            Case {
                source: "{ if structure { a: 1 } {} }",
                expect: concat!(
                    "{\n",
                    "    if structure {\n",
                    "        Error\n",
                    "    }\n",
                    "    {\n",
                    "    }\n",
                    "}",
                ),
                errors: 1,
            },
            Case {
                source: "{ if ( structure { a: 1 } ) {} }",
                expect: concat!("{\n", "    if ((structure { a: 1 })) {\n", "    }\n", "}",),
                errors: 0,
            },
            Case {
                source: "{ if ( structure {} ) {} }",
                expect: concat!("{\n", "    if ((structure {  })) {\n", "    }\n", "}"),
                errors: 0,
            },
            Case {
                source: "{ if (a { x: 1 }, b { y: 2 }) {} }",
                expect: concat!("{\n", "    if ((a { x: 1 }), (b { y: 2 })) {\n", "    }\n", "}",),
                errors: 0,
            },
            Case {
                source: "{ if ({ let foo = bar { baz: 42 }; foo == bar { baz: 42 }}) {} }",
                expect: concat!(
                    "{\n",
                    "    if ({\n",
                    "        let foo: unspecified = (bar { baz: 42 })\n",
                    "        (foo == (bar { baz: 42 }))\n",
                    "    }) {\n",
                    "    }\n",
                    "}",
                ),
                errors: 0,
            },
        ];

        check_cases_with_errors(&cases[..], block(fresh_statement()));
    }

    #[test]
    fn test_quote() {
        let cases = vec![
            "quote {}",
            "quote { a.b }",
            "quote { ) ( }", // invalid syntax is fine in a quote
            "quote { { } }", // Nested `{` and `}` shouldn't close the quote as long as they are matched.
            "quote { 1 { 2 { 3 { 4 { 5 } 4 4 } 3 3 } 2 2 } 1 1 }",
        ];
        parse_all(quote(), cases);

        let failing = vec!["quote {}}", "quote a", "quote { { { } } } }"];
        parse_all_failing(quote(), failing);
    }

    #[test]
    fn test_parses_block_statement_not_infix_expression() {
        let src = r#"
        {
          {}
          -1
        }"#;
        let (block_expr, _) = parse_recover(block(fresh_statement()), src);
        let block_expr = block_expr.expect("Failed to parse module");
        assert_eq!(block_expr.statements.len(), 2);
    }

    #[test]
    fn test_parses_if_statement_not_infix_expression() {
        let src = r#"
        {
          if 1 { 2 } else { 3 }
          -1
        }"#;
        let (block_expr, _) = parse_recover(block(fresh_statement()), src);
        let block_expr = block_expr.expect("Failed to parse module");
        assert_eq!(block_expr.statements.len(), 2);
    }

    #[test]
    fn test_parses_if_statement_followed_by_tuple_as_two_separate_statements() {
        // Regression for #1310: this should not be parsed as a function call
        let src = r#"
        {
          if 1 { 2 } else { 3 } (1, 2)
        }"#;
        let (block_expr, _) = parse_recover(block(fresh_statement()), src);
        let block_expr = block_expr.expect("Failed to parse module");
        assert_eq!(block_expr.statements.len(), 2);
    }

    #[test]
    fn test_parses_member_access_without_member_name() {
        let src = "{ foo. }";

        let (Some(block_expression), errors) = parse_recover(block(fresh_statement()), src) else {
            panic!("Expected to be able to parse a block expression");
        };

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "expected an identifier after .");

        let statement = &block_expression.statements[0];
        let StatementKind::Expression(expr) = &statement.kind else {
            panic!("Expected an expression statement");
        };

        let ExpressionKind::Variable(var) = &expr.kind else {
            panic!("Expected a variable expression");
        };

        assert_eq!(var.to_string(), "foo");
    }

    #[test]
    fn parse_recover_impl_without_body() {
        let src = "impl Foo";

        let (top_level_statement, errors) = parse_recover(implementation(), src);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "expected <, where or { after impl type");

        let top_level_statement = top_level_statement.unwrap();
        let TopLevelStatement::Impl(impl_) = top_level_statement else {
            panic!("Expected to parse an impl");
        };

        assert_eq!(impl_.object_type.to_string(), "Foo");
        assert!(impl_.methods.is_empty());
    }
}
