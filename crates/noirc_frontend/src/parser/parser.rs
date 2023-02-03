use super::{
    foldl_with_span, parameter_name_recovery, parameter_recovery, parenthesized, then_commit,
    then_commit_ignore, top_level_statement_recovery, ExprParser, ForRange, NoirParser,
    ParsedModule, ParserError, Precedence, SubModule, TopLevelStatement,
};
use crate::ast::{Expression, ExpressionKind, LetStatement, Statement, UnresolvedType};
use crate::lexer::Lexer;
use crate::parser::{force, ignore_then_commit, statement_recovery};
use crate::token::{Attribute, Keyword, Token, TokenKind};
use crate::{
    BinaryOp, BinaryOpKind, BlockExpression, CompTime, ConstrainStatement, FunctionDefinition,
    Ident, IfExpression, ImportStatement, InfixExpression, LValue, Lambda, NoirFunction, NoirImpl,
    NoirStruct, Path, PathKind, Pattern, Recoverable, UnaryOp, UnresolvedTypeExpression,
};

use chumsky::prelude::*;
use iter_extended::vecmap;
use noirc_abi::AbiVisibility;
use noirc_errors::{CustomDiagnostic, DiagnosableError, Span, Spanned};

pub fn parse_program(source_program: &str) -> (ParsedModule, Vec<CustomDiagnostic>) {
    let lexer = Lexer::new(source_program);
    let (tokens, lexing_errors) = lexer.lex();
    let mut errors = vecmap(&lexing_errors, DiagnosableError::to_diagnostic);

    let (module, parsing_errors) = program().parse_recovery_verbose(tokens);
    errors.extend(parsing_errors.iter().map(DiagnosableError::to_diagnostic));

    (module.unwrap(), errors)
}

fn program() -> impl NoirParser<ParsedModule> {
    module().then_ignore(force(just(Token::EOF)))
}

fn module() -> impl NoirParser<ParsedModule> {
    recursive(|module_parser| {
        empty()
            .map(|_| ParsedModule::default())
            .then(top_level_statement(module_parser).repeated())
            .foldl(|mut program, statement| {
                match statement {
                    TopLevelStatement::Function(f) => program.push_function(f),
                    TopLevelStatement::Module(m) => program.push_module_decl(m),
                    TopLevelStatement::Import(i) => program.push_import(i),
                    TopLevelStatement::Struct(s) => program.push_type(s),
                    TopLevelStatement::Impl(i) => program.push_impl(i),
                    TopLevelStatement::SubModule(s) => program.push_submodule(s),
                    TopLevelStatement::Global(c) => program.push_global(c),
                    TopLevelStatement::Error => (),
                }
                program
            })
    })
}

fn top_level_statement(
    module_parser: impl NoirParser<ParsedModule>,
) -> impl NoirParser<TopLevelStatement> {
    choice((
        function_definition(false).map(TopLevelStatement::Function),
        struct_definition(),
        implementation(),
        submodule(module_parser),
        module_declaration().then_ignore(force(just(Token::Semicolon))),
        use_statement().then_ignore(force(just(Token::Semicolon))),
        global_declaration().then_ignore(force(just(Token::Semicolon))),
    ))
    .recover_via(top_level_statement_recovery())
}

fn global_declaration() -> impl NoirParser<TopLevelStatement> {
    let p = ignore_then_commit(
        keyword(Keyword::Global).labelled("global"),
        ident().map(Pattern::Identifier),
    );
    let p = then_commit(p, global_type_annotation());
    let p = then_commit_ignore(p, just(Token::Assign));
    let p = then_commit(p, literal().map_with_span(Expression::new)); // XXX: this should be a literal
    p.map(LetStatement::new_let).map(TopLevelStatement::Global)
}

fn submodule(module_parser: impl NoirParser<ParsedModule>) -> impl NoirParser<TopLevelStatement> {
    keyword(Keyword::Mod)
        .ignore_then(ident())
        .then_ignore(just(Token::LeftBrace))
        .then(module_parser)
        .then_ignore(just(Token::RightBrace))
        .map(|(name, contents)| TopLevelStatement::SubModule(SubModule { name, contents }))
}

fn function_definition(allow_self: bool) -> impl NoirParser<NoirFunction> {
    attribute()
        .or_not()
        .then_ignore(keyword(Keyword::Fn))
        .then(ident())
        .then(generics())
        .then(parenthesized(function_parameters(allow_self)))
        .then(function_return_type())
        .then(block(expression()))
        .map(
            |(
                ((((attribute, name), generics), parameters), (return_visibility, return_type)),
                body,
            )| {
                FunctionDefinition {
                    span: name.0.span(),
                    name,
                    attribute, // XXX: Currently we only have one attribute defined. If more attributes are needed per function, we can make this a vector and make attribute definition more expressive
                    generics,
                    parameters,
                    body,
                    return_type,
                    return_visibility,
                }
                .into()
            },
        )
}

fn generics() -> impl NoirParser<Vec<Ident>> {
    ident()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .at_least(1)
        .delimited_by(just(Token::Less), just(Token::Greater))
        .or_not()
        .map(|opt| opt.unwrap_or_default())
}

fn struct_definition() -> impl NoirParser<TopLevelStatement> {
    use self::Keyword::Struct;
    use Token::*;

    let fields = struct_fields().delimited_by(just(LeftBrace), just(RightBrace)).recover_with(
        nested_delimiters(
            LeftBrace,
            RightBrace,
            [(LeftParen, RightParen), (LeftBracket, RightBracket)],
            |_| vec![],
        ),
    );

    keyword(Struct).ignore_then(ident()).then(generics()).then(fields).map_with_span(
        |((name, generics), fields), span| {
            TopLevelStatement::Struct(NoirStruct { name, generics, fields, span })
        },
    )
}

fn lambda_return_type() -> impl NoirParser<UnresolvedType> {
    just(Token::Arrow)
        .ignore_then(parse_type())
        .or_not()
        .map(|ret| ret.unwrap_or(UnresolvedType::Unspecified))
}

fn function_return_type() -> impl NoirParser<(AbiVisibility, UnresolvedType)> {
    just(Token::Arrow)
        .ignore_then(optional_visibility())
        .then(parse_type())
        .or_not()
        .map(|ret| ret.unwrap_or((AbiVisibility::Private, UnresolvedType::Unit)))
}

fn attribute() -> impl NoirParser<Attribute> {
    tokenkind(TokenKind::Attribute).map(|token| match token {
        Token::Attribute(attribute) => attribute,
        _ => unreachable!(),
    })
}

fn struct_fields() -> impl NoirParser<Vec<(Ident, UnresolvedType)>> {
    ident()
        .then_ignore(just(Token::Colon))
        .then(parse_type())
        .separated_by(just(Token::Comma))
        .allow_trailing()
}

fn lambda_parameters() -> impl NoirParser<Vec<(Pattern, UnresolvedType)>> {
    let typ = parse_type().recover_via(parameter_recovery());
    let typ = just(Token::Colon).ignore_then(typ);

    let parameter = pattern()
        .recover_via(parameter_name_recovery())
        .then(typ.or_not().map(|typ| typ.unwrap_or(UnresolvedType::Unspecified)));

    parameter.separated_by(just(Token::Comma)).allow_trailing().labelled("parameter")
}

fn function_parameters<'a>(
    allow_self: bool,
) -> impl NoirParser<Vec<(Pattern, UnresolvedType, AbiVisibility)>> + 'a {
    let typ = parse_type().recover_via(parameter_recovery());

    let full_parameter = pattern()
        .recover_via(parameter_name_recovery())
        .then_ignore(just(Token::Colon))
        .then(optional_visibility())
        .then(typ)
        .map(|((name, visibility), typ)| (name, typ, visibility));

    let self_parameter = if allow_self { self_parameter().boxed() } else { nothing().boxed() };

    let parameter = full_parameter.or(self_parameter);

    parameter.separated_by(just(Token::Comma)).allow_trailing().labelled("parameter")
}

/// This parser always parses no input and fails
fn nothing<T>() -> impl NoirParser<T> {
    one_of([]).map(|_| unreachable!())
}

fn self_parameter() -> impl NoirParser<(Pattern, UnresolvedType, AbiVisibility)> {
    filter_map(move |span, found: Token| match found {
        Token::Ident(ref word) if word == "self" => {
            let ident = Ident::from_token(found, span);
            let path = Path::from_single("Self".to_owned(), span);
            let self_type = UnresolvedType::Named(path, vec![]);
            Ok((Pattern::Identifier(ident), self_type, AbiVisibility::Private))
        }
        _ => Err(ParserError::expected_label("parameter".to_owned(), found, span)),
    })
}

fn implementation() -> impl NoirParser<TopLevelStatement> {
    keyword(Keyword::Impl)
        .ignore_then(path())
        .then_ignore(just(Token::LeftBrace))
        .then(function_definition(true).repeated())
        .then_ignore(just(Token::RightBrace))
        .map(|(type_path, methods)| TopLevelStatement::Impl(NoirImpl { type_path, methods }))
}

fn block_expr<'a, P>(expr_parser: P) -> impl NoirParser<Expression> + 'a
where
    P: ExprParser + 'a,
{
    block(expr_parser).map(ExpressionKind::Block).map_with_span(Expression::new)
}

fn block<'a, P>(expr_parser: P) -> impl NoirParser<BlockExpression> + 'a
where
    P: ExprParser + 'a,
{
    use Token::*;
    statement(expr_parser)
        .recover_via(statement_recovery())
        .then(just(Semicolon).or_not().map_with_span(|s, span| (s, span)))
        .repeated()
        .validate(check_statements_require_semicolon)
        .delimited_by(just(LeftBrace), just(RightBrace))
        .recover_with(nested_delimiters(
            LeftBrace,
            RightBrace,
            [(LeftParen, RightParen), (LeftBracket, RightBracket)],
            |_| vec![Statement::Error],
        ))
        .map(BlockExpression)
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

/// Parse an optional ': type' and implicitly add a 'comptime' to the type
fn global_type_annotation() -> impl NoirParser<UnresolvedType> {
    ignore_then_commit(just(Token::Colon), parse_type())
        .map(|r#type| match r#type {
            UnresolvedType::FieldElement(_) => UnresolvedType::FieldElement(CompTime::Yes(None)),
            UnresolvedType::Bool(_) => UnresolvedType::Bool(CompTime::Yes(None)),
            UnresolvedType::Integer(_, sign, size) => {
                UnresolvedType::Integer(CompTime::Yes(None), sign, size)
            }
            other => other,
        })
        .or_not()
        .map(|opt| opt.unwrap_or(UnresolvedType::Unspecified))
}

fn optional_type_annotation<'a>() -> impl NoirParser<UnresolvedType> + 'a {
    ignore_then_commit(just(Token::Colon), parse_type())
        .or_not()
        .map(|r#type| r#type.unwrap_or(UnresolvedType::Unspecified))
}

fn module_declaration() -> impl NoirParser<TopLevelStatement> {
    keyword(Keyword::Mod).ignore_then(ident()).map(TopLevelStatement::Module)
}

fn use_statement() -> impl NoirParser<TopLevelStatement> {
    let rename = ignore_then_commit(keyword(Keyword::As), ident()).or_not();

    keyword(Keyword::Use)
        .ignore_then(path())
        .then(rename)
        .map(|(path, alias)| TopLevelStatement::Import(ImportStatement { path, alias }))
}

fn keyword(keyword: Keyword) -> impl NoirParser<Token> {
    just(Token::Keyword(keyword))
}

fn tokenkind(tokenkind: TokenKind) -> impl NoirParser<Token> {
    filter_map(move |span, found: Token| {
        if found.kind() == tokenkind {
            Ok(found)
        } else {
            Err(ParserError::expected_label(tokenkind.to_string(), found, span))
        }
    })
}

fn path() -> impl NoirParser<Path> {
    let idents = || ident().separated_by(just(Token::DoubleColon)).at_least(1);
    let make_path = |kind| move |segments| Path { segments, kind };

    let prefix = |key| keyword(key).ignore_then(just(Token::DoubleColon));
    let path_kind = |key, kind| prefix(key).ignore_then(idents()).map(make_path(kind));

    choice((
        path_kind(Keyword::Crate, PathKind::Crate),
        path_kind(Keyword::Dep, PathKind::Dep),
        idents().map(make_path(PathKind::Plain)),
    ))
}

fn ident() -> impl NoirParser<Ident> {
    tokenkind(TokenKind::Ident).map_with_span(Ident::from_token)
}

fn statement<'a, P>(expr_parser: P) -> impl NoirParser<Statement> + 'a
where
    P: ExprParser + 'a,
{
    choice((
        constrain(expr_parser.clone()),
        declaration(expr_parser.clone()),
        assignment(expr_parser.clone()),
        expr_parser.map(Statement::Expression),
    ))
}

fn constrain<'a, P>(expr_parser: P) -> impl NoirParser<Statement> + 'a
where
    P: ExprParser + 'a,
{
    ignore_then_commit(keyword(Keyword::Constrain).labelled("statement"), expr_parser)
        .map(|expr| Statement::Constrain(ConstrainStatement(expr)))
}

fn declaration<'a, P>(expr_parser: P) -> impl NoirParser<Statement> + 'a
where
    P: ExprParser + 'a,
{
    let p = ignore_then_commit(keyword(Keyword::Let).labelled("statement"), pattern());
    let p = p.then(optional_type_annotation());
    let p = then_commit_ignore(p, just(Token::Assign));
    let p = then_commit(p, expr_parser);
    p.map(Statement::new_let)
}

fn pattern() -> impl NoirParser<Pattern> {
    recursive(|pattern| {
        let ident_pattern = ident().map(Pattern::Identifier);

        let mut_pattern = keyword(Keyword::Mut)
            .ignore_then(pattern.clone())
            .map_with_span(|inner, span| Pattern::Mutable(Box::new(inner), span));

        let short_field = ident().map(|name| (name.clone(), Pattern::Identifier(name)));
        let long_field = ident().then_ignore(just(Token::Colon)).then(pattern.clone());

        let struct_pattern_fields = long_field
            .or(short_field)
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::LeftBrace), just(Token::RightBrace));

        let struct_pattern = path()
            .then(struct_pattern_fields)
            .map_with_span(|(typename, fields), span| Pattern::Struct(typename, fields, span));

        let tuple_pattern = pattern
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::LeftParen), just(Token::RightParen))
            .map_with_span(Pattern::Tuple);

        choice((mut_pattern, tuple_pattern, struct_pattern, ident_pattern))
    })
    .labelled("pattern")
}

fn assignment<'a, P>(expr_parser: P) -> impl NoirParser<Statement> + 'a
where
    P: ExprParser + 'a,
{
    let failable = lvalue(expr_parser.clone()).then(assign_operator()).labelled("statement");

    then_commit(failable, expr_parser).map_with_span(
        |((identifier, operator), expression), span| {
            Statement::assign(identifier, operator, expression, span)
        },
    )
}

fn assign_operator() -> impl NoirParser<Token> {
    let shorthand_operators = Token::assign_shorthand_operators();
    let shorthand_syntax = one_of(shorthand_operators).then_ignore(just(Token::Assign));
    just(Token::Assign).or(shorthand_syntax)
}

enum LValueRhs {
    MemberAccess(Ident),
    Index(Expression),
}

fn lvalue<'a, P>(expr_parser: P) -> impl NoirParser<LValue>
where
    P: ExprParser + 'a,
{
    let l_ident = ident().map(LValue::Ident);

    let l_member_rhs = just(Token::Dot).ignore_then(ident()).map(LValueRhs::MemberAccess);

    let l_index = expr_parser
        .delimited_by(just(Token::LeftBracket), just(Token::RightBracket))
        .map(LValueRhs::Index);

    l_ident.then(l_member_rhs.or(l_index).repeated()).foldl(|lvalue, rhs| match rhs {
        LValueRhs::MemberAccess(field_name) => {
            LValue::MemberAccess { object: Box::new(lvalue), field_name }
        }
        LValueRhs::Index(index) => LValue::Index { array: Box::new(lvalue), index },
    })
}

fn parse_type<'a>() -> impl NoirParser<UnresolvedType> + 'a {
    recursive(parse_type_inner)
}

fn parse_type_inner(
    recursive_type_parser: impl NoirParser<UnresolvedType>,
) -> impl NoirParser<UnresolvedType> {
    choice((
        field_type(),
        int_type(),
        named_type(recursive_type_parser.clone()),
        array_type(recursive_type_parser.clone()),
        tuple_type(recursive_type_parser.clone()),
        bool_type(),
        string_type(),
        function_type(recursive_type_parser),
    ))
}

fn optional_visibility() -> impl NoirParser<AbiVisibility> {
    keyword(Keyword::Pub).or_not().map(|opt| match opt {
        Some(_) => AbiVisibility::Public,
        None => AbiVisibility::Private,
    })
}

fn maybe_comptime() -> impl NoirParser<CompTime> {
    keyword(Keyword::CompTime).or_not().map(|opt| match opt {
        Some(_) => CompTime::Yes(None),
        None => CompTime::No(None),
    })
}

fn field_type() -> impl NoirParser<UnresolvedType> {
    maybe_comptime().then_ignore(keyword(Keyword::Field)).map(UnresolvedType::FieldElement)
}

fn bool_type() -> impl NoirParser<UnresolvedType> {
    maybe_comptime().then_ignore(keyword(Keyword::Bool)).map(UnresolvedType::Bool)
}

fn string_type() -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::String)
        .ignore_then(
            type_expression().delimited_by(just(Token::Less), just(Token::Greater)).or_not(),
        )
        .map(UnresolvedType::String)
}

fn int_type() -> impl NoirParser<UnresolvedType> {
    maybe_comptime()
        .then(filter_map(|span, token: Token| match token {
            Token::IntType(int_type) => Ok(int_type),
            unexpected => {
                Err(ParserError::expected_label("integer type".to_string(), unexpected, span))
            }
        }))
        .map(UnresolvedType::from_int_token)
}

fn named_type(type_parser: impl NoirParser<UnresolvedType>) -> impl NoirParser<UnresolvedType> {
    path()
        .then(generic_type_args(type_parser))
        .map(|(path, args)| UnresolvedType::Named(path, args))
}

fn generic_type_args(
    type_parser: impl NoirParser<UnresolvedType>,
) -> impl NoirParser<Vec<UnresolvedType>> {
    type_parser
        // Without checking for a terminating ',' or '>' here we may incorrectly
        // parse a generic `N * 2` as just the type `N` then fail when there is no
        // separator afterward. Failing early here ensures we try the `type_expression`
        // parser afterward.
        .then_ignore(one_of([Token::Comma, Token::Greater]).rewind())
        .or(type_expression().map(UnresolvedType::Expression))
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .at_least(1)
        .delimited_by(just(Token::Less), just(Token::Greater))
        .or_not()
        .map(Option::unwrap_or_default)
}

fn array_type(type_parser: impl NoirParser<UnresolvedType>) -> impl NoirParser<UnresolvedType> {
    just(Token::LeftBracket)
        .ignore_then(type_parser)
        .then(just(Token::Semicolon).ignore_then(type_expression()).or_not())
        .then_ignore(just(Token::RightBracket))
        .map(|(element_type, size)| UnresolvedType::Array(size, Box::new(element_type)))
}

fn type_expression() -> impl NoirParser<UnresolvedTypeExpression> {
    recursive(|expr| expression_with_precedence(Precedence::lowest_type_precedence(), expr, true))
        .labelled("type expression")
        .try_map(UnresolvedTypeExpression::from_expr)
}

fn tuple_type<T>(type_parser: T) -> impl NoirParser<UnresolvedType>
where
    T: NoirParser<UnresolvedType>,
{
    let fields = type_parser.separated_by(just(Token::Comma)).allow_trailing();
    parenthesized(fields).map(UnresolvedType::Tuple)
}

fn function_type<T>(type_parser: T) -> impl NoirParser<UnresolvedType>
where
    T: NoirParser<UnresolvedType>,
{
    let args = parenthesized(type_parser.clone().separated_by(just(Token::Comma)).allow_trailing());
    keyword(Keyword::Fn)
        .ignore_then(args)
        .then_ignore(just(Token::Arrow))
        .then(type_parser)
        .map(|(args, ret)| UnresolvedType::Function(args, Box::new(ret)))
}

fn expression() -> impl ExprParser {
    recursive(|expr| expression_with_precedence(Precedence::Lowest, expr, false))
        .labelled("expression")
}

// An expression is a single term followed by 0 or more (OP subexpression)*
// where OP is an operator at the given precedence level and subexpression
// is an expression at the current precedence level plus one.
fn expression_with_precedence<'a, P>(
    precedence: Precedence,
    expr_parser: P,
    // True if we should only parse the restricted subset of operators valid within type expressions
    is_type_expression: bool,
) -> impl NoirParser<Expression> + 'a
where
    P: ExprParser + 'a,
{
    if precedence == Precedence::Highest {
        if is_type_expression {
            type_expression_term(expr_parser).boxed().labelled("term")
        } else {
            term(expr_parser).boxed().labelled("term")
        }
    } else {
        let next_precedence =
            if is_type_expression { precedence.next_type_precedence() } else { precedence.next() };

        expression_with_precedence(precedence.next(), expr_parser.clone(), is_type_expression)
            .then(
                then_commit(
                    operator_with_precedence(precedence),
                    expression_with_precedence(next_precedence, expr_parser, is_type_expression),
                )
                .repeated(),
            )
            .foldl(create_infix_expression)
            .boxed()
            .labelled("expression")
    }
}

fn create_infix_expression(lhs: Expression, (operator, rhs): (BinaryOp, Expression)) -> Expression {
    let span = lhs.span.merge(rhs.span);
    let infix = Box::new(InfixExpression { lhs, operator, rhs });

    Expression { span, kind: ExpressionKind::Infix(infix) }
}

fn operator_with_precedence(precedence: Precedence) -> impl NoirParser<Spanned<BinaryOpKind>> {
    filter_map(move |span, token: Token| {
        if Precedence::token_precedence(&token) == Some(precedence) {
            Ok(token.try_into_binop(span).unwrap())
        } else {
            Err(ParserError::expected_label("binary operator".to_string(), token, span))
        }
    })
}

fn term<'a, P>(expr_parser: P) -> impl NoirParser<Expression> + 'a
where
    P: ExprParser + 'a,
{
    recursive(move |term_parser| {
        choice((not(term_parser.clone()), negation(term_parser)))
            .map_with_span(Expression::new)
            // right-unary operators like a[0] or a.f bind more tightly than left-unary
            // operators like  - or !, so that !a[0] is parsed as !(a[0]). This is a bit
            // awkward for casts so -a as i32 actually binds as -(a as i32).
            .or(atom_or_right_unary(expr_parser))
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

fn atom_or_right_unary<'a, P>(expr_parser: P) -> impl NoirParser<Expression> + 'a
where
    P: ExprParser + 'a,
{
    enum UnaryRhs {
        Call(Vec<Expression>),
        ArrayIndex(Expression),
        Cast(UnresolvedType),
        MemberAccess((Ident, Option<Vec<Expression>>)),
    }

    // `(arg1, ..., argN)` in `my_func(arg1, ..., argN)`
    let call_rhs = parenthesized(expression_list(expr_parser.clone())).map(UnaryRhs::Call);

    // `[expr]` in `arr[expr]`
    let array_rhs = expr_parser
        .clone()
        .delimited_by(just(Token::LeftBracket), just(Token::RightBracket))
        .map(UnaryRhs::ArrayIndex);

    // `as Type` in `atom as Type`
    let cast_rhs =
        keyword(Keyword::As).ignore_then(parse_type()).map(UnaryRhs::Cast).labelled("cast");

    // `.foo` or `.foo(args)` in `atom.foo` or `atom.foo(args)`
    let member_rhs = just(Token::Dot)
        .ignore_then(field_name())
        .then(parenthesized(expression_list(expr_parser.clone())).or_not())
        .map(UnaryRhs::MemberAccess)
        .labelled("field access");

    let rhs = choice((call_rhs, array_rhs, cast_rhs, member_rhs));

    foldl_with_span(atom(expr_parser), rhs, |lhs, rhs, span| match rhs {
        UnaryRhs::Call(args) => Expression::call(lhs, args, span),
        UnaryRhs::ArrayIndex(index) => Expression::index(lhs, index, span),
        UnaryRhs::Cast(r#type) => Expression::cast(lhs, r#type, span),
        UnaryRhs::MemberAccess(field) => Expression::member_access_or_method_call(lhs, field, span),
    })
}

fn if_expr<'a, P>(expr_parser: P) -> impl NoirParser<ExpressionKind> + 'a
where
    P: ExprParser + 'a,
{
    recursive(|if_parser| {
        let if_block = block_expr(expr_parser.clone());
        // The else block could also be an `else if` block, in which case we must recursively parse it.
        let else_block =
            block_expr(expr_parser.clone()).or(if_parser.map_with_span(|kind, span| {
                // Wrap the inner `if` expression in a block expression.
                // i.e. rewrite the sugared form `if cond1 {} else if cond2 {}` as `if cond1 {} else { if cond2 {} }`.
                let if_expression = Expression::new(kind, span);
                let desugared_else = BlockExpression(vec![Statement::Expression(if_expression)]);
                Expression::new(ExpressionKind::Block(desugared_else), span)
            }));

        keyword(Keyword::If)
            .ignore_then(expr_parser)
            .then(if_block)
            .then(keyword(Keyword::Else).ignore_then(else_block).or_not())
            .map(|((condition, consequence), alternative)| {
                ExpressionKind::If(Box::new(IfExpression { condition, consequence, alternative }))
            })
    })
}

fn lambda<'a>(
    expr_parser: impl NoirParser<Expression> + 'a,
) -> impl NoirParser<ExpressionKind> + 'a {
    lambda_parameters()
        .delimited_by(just(Token::Pipe), just(Token::Pipe))
        .then(lambda_return_type())
        .then(expr_parser)
        .map(|((parameters, return_type), body)| {
            ExpressionKind::Lambda(Box::new(Lambda { parameters, return_type, body }))
        })
}

fn for_expr<'a, P>(expr_parser: P) -> impl NoirParser<ExpressionKind> + 'a
where
    P: ExprParser + 'a,
{
    keyword(Keyword::For)
        .ignore_then(ident())
        .then_ignore(keyword(Keyword::In))
        .then(for_range(expr_parser.clone()))
        .then(block_expr(expr_parser))
        .map_with_span(|((identifier, range), block), span| range.into_for(identifier, block, span))
}

/// The 'range' of a for loop. Either an actual range `start .. end` or an array expression.
fn for_range<P>(expr_parser: P) -> impl NoirParser<ForRange>
where
    P: ExprParser,
{
    expr_parser
        .clone()
        .then_ignore(just(Token::DoubleDot))
        .then(expr_parser.clone())
        .map(|(start, end)| ForRange::Range(start, end))
        .or(expr_parser.map(ForRange::Array))
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
        .validate(|elems, span, emit| {
            if elems.is_empty() {
                emit(ParserError::with_reason(
                    "Arrays must have at least one element".to_owned(),
                    span,
                ))
            }
            ExpressionKind::array(elems)
        })
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

fn expression_list<P>(expr_parser: P) -> impl NoirParser<Vec<Expression>>
where
    P: ExprParser,
{
    expr_parser.separated_by(just(Token::Comma)).allow_trailing()
}

fn not<P>(term_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    just(Token::Bang).ignore_then(term_parser).map(|rhs| ExpressionKind::prefix(UnaryOp::Not, rhs))
}

fn negation<P>(term_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    just(Token::Minus)
        .ignore_then(term_parser)
        .map(|rhs| ExpressionKind::prefix(UnaryOp::Minus, rhs))
}

fn atom<'a, P>(expr_parser: P) -> impl NoirParser<Expression> + 'a
where
    P: ExprParser + 'a,
{
    choice((
        if_expr(expr_parser.clone()),
        for_expr(expr_parser.clone()),
        array_expr(expr_parser.clone()),
        constructor(expr_parser.clone()),
        lambda(expr_parser.clone()),
        block(expr_parser.clone()).map(ExpressionKind::Block),
        variable(),
        literal(),
    ))
    .map_with_span(Expression::new)
    .or(parenthesized(expr_parser.clone()))
    .or(tuple(expr_parser))
    .labelled("atom")
}

/// Atoms within type expressions are limited to only variables, literals, and parenthesized
/// type expressions.
fn type_expression_atom<'a, P>(expr_parser: P) -> impl NoirParser<Expression> + 'a
where
    P: ExprParser + 'a,
{
    variable()
        .or(literal())
        .map_with_span(Expression::new)
        .or(parenthesized(expr_parser))
        .labelled("atom")
}

fn tuple<P>(expr_parser: P) -> impl NoirParser<Expression>
where
    P: ExprParser,
{
    parenthesized(expression_list(expr_parser))
        .map_with_span(|elements, span| Expression::new(ExpressionKind::Tuple(elements), span))
}

fn field_name() -> impl NoirParser<Ident> {
    ident().or(tokenkind(TokenKind::Literal).validate(|token, span, emit| match token {
        Token::Int(_) => Ident::from(Spanned::from(span, token.to_string())),
        other => {
            let reason = format!("Unexpected '{other}', expected a field name");
            emit(ParserError::with_reason(reason, span));
            Ident::error(span)
        }
    }))
}

fn constructor<P>(expr_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    let args = constructor_field(expr_parser)
        .separated_by(just(Token::Comma))
        .at_least(1)
        .allow_trailing()
        .delimited_by(just(Token::LeftBrace), just(Token::RightBrace));

    path().then(args).map(ExpressionKind::constructor)
}

fn constructor_field<P>(expr_parser: P) -> impl NoirParser<(Ident, Expression)>
where
    P: ExprParser,
{
    let long_form = ident().then_ignore(just(Token::Colon)).then(expr_parser);
    let short_form = ident().map(|ident| (ident.clone(), ident.into()));
    long_form.or(short_form)
}

fn variable() -> impl NoirParser<ExpressionKind> {
    path().map(ExpressionKind::Variable)
}

fn literal() -> impl NoirParser<ExpressionKind> {
    tokenkind(TokenKind::Literal).map(|token| match token {
        Token::Int(x) => ExpressionKind::integer(x),
        Token::Bool(b) => ExpressionKind::boolean(b),
        Token::Str(s) => ExpressionKind::string(s),
        unexpected => unreachable!("Non-literal {} parsed as a literal", unexpected),
    })
}

#[cfg(test)]
mod test {
    use noirc_errors::{CustomDiagnostic, DiagnosableError};

    use super::*;
    use crate::{ArrayLiteral, Literal};

    fn parse_with<P, T>(parser: P, program: &str) -> Result<T, Vec<CustomDiagnostic>>
    where
        P: NoirParser<T>,
    {
        let lexer = Lexer::new(program);
        let (tokens, lexer_errors) = lexer.lex();
        if !lexer_errors.is_empty() {
            return Err(vecmap(&lexer_errors, DiagnosableError::to_diagnostic));
        }
        parser
            .then_ignore(just(Token::EOF))
            .parse(tokens)
            .map_err(|errors| vecmap(&errors, DiagnosableError::to_diagnostic))
    }

    fn parse_recover<P, T>(parser: P, program: &str) -> (Option<T>, Vec<CustomDiagnostic>)
    where
        P: NoirParser<T>,
    {
        let lexer = Lexer::new(program);
        let (tokens, lexer_errors) = lexer.lex();
        let (opt, errs) = parser.then_ignore(force(just(Token::EOF))).parse_recovery(tokens);

        let mut errors = vecmap(&lexer_errors, DiagnosableError::to_diagnostic);
        errors.extend(errs.iter().map(DiagnosableError::to_diagnostic));

        (opt, errors)
    }

    fn parse_all<P, T>(parser: P, programs: Vec<&str>) -> Vec<T>
    where
        P: NoirParser<T>,
    {
        vecmap(programs, move |program| {
            let message = format!("Failed to parse:\n{}", program);
            parse_with(&parser, program).expect(&message)
        })
    }

    fn parse_all_failing<P, T>(parser: P, programs: Vec<&str>) -> Vec<CustomDiagnostic>
    where
        P: NoirParser<T>,
        T: std::fmt::Display,
    {
        programs
            .into_iter()
            .flat_map(|program| match parse_with(&parser, program) {
                Ok(expr) => unreachable!(
                    "Expected this input to fail:\n{}\nYet it successfully parsed as:\n{}",
                    program, expr
                ),
                Err(error) => error,
            })
            .collect()
    }

    #[test]
    fn regression_skip_comment() {
        parse_all(
            function_definition(false),
            vec![
                "fn main(
                // This comment should be skipped
                x : Field,
                // And this one
                y : Field,
            ) {
            }",
                "fn main(x : Field, y : Field,) {
                foo::bar(
                    // Comment for x argument
                    x,
                    // Comment for y argument
                    y
                )
            }",
            ],
        );
    }

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
        parse_all(
            atom_or_right_unary(expression()),
            vec!["x as u8", "0 as Field", "(x + 3) as [Field; 8]"],
        );
        parse_all_failing(atom_or_right_unary(expression()), vec!["x as pub u8"]);
    }

    #[test]
    fn parse_array_index() {
        let valid = vec![
            "x[9]",
            "y[x+a]",
            " foo [foo+5]",
            "baz[bar]",
            "foo.bar[3] as Field .baz as i32 [7]",
        ];
        parse_all(atom_or_right_unary(expression()), valid);
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
                ArrayLiteral::Standard(elems) => assert_eq!(elems.len(), 5),
                ArrayLiteral::Repeated { length, .. } => {
                    assert_eq!(length.kind, ExpressionKind::integer(5i128.into()))
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

    #[test]
    fn parse_block() {
        parse_with(block(expression()), "{ [0,1,2,3,4] }").unwrap();

        parse_all_failing(
            block(expression()),
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

    /// This is the standard way to declare a constrain statement
    #[test]
    fn parse_constrain() {
        parse_with(constrain(expression()), "constrain x == y").unwrap();

        // Currently we disallow constrain statements where the outer infix operator
        // produces a value. This would require an implicit `==` which
        // may not be intuitive to the user.
        //
        // If this is deemed useful, one would either apply a transformation
        // or interpret it with an `==` in the evaluator
        let disallowed_operators = vec![
            BinaryOpKind::And,
            BinaryOpKind::Subtract,
            BinaryOpKind::Divide,
            BinaryOpKind::Multiply,
            BinaryOpKind::Or,
        ];

        for operator in disallowed_operators {
            let src = format!("constrain x {} y;", operator.as_string());
            parse_with(constrain(expression()), &src).unwrap_err();
        }

        // These are general cases which should always work.
        //
        // The first case is the most noteworthy. It contains two `==`
        // The first (inner) `==` is a predicate which returns 0/1
        // The outer layer is an infix `==` which is
        // associated with the Constrain statement
        parse_all(
            constrain(expression()),
            vec![
                "constrain ((x + y) == k) + z == y",
                "constrain (x + !y) == y",
                "constrain (x ^ y) == y",
                "constrain (x ^ y) == (y + m)",
                "constrain x + x ^ x == y | m",
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
        parse_all(declaration(expression()), vec!["let x = y", "let x : u8 = y"]);
    }

    #[test]
    fn parse_invalid_pub() {
        // pub cannot be used to declare a statement
        parse_all_failing(statement(expression()), vec!["pub x = y", "pub x : pub Field = y"]);
    }

    #[test]
    fn parse_for_loop() {
        parse_all(
            for_expr(expression()),
            vec!["for i in x+y..z {}", "for i in 0..100 { foo; bar }"],
        );

        parse_all_failing(
            for_expr(expression()),
            vec![
                "for 1 in x+y..z {}",  // Cannot have a literal as the loop identifier
                "for i in 0...100 {}", // Only '..' is supported, there are no inclusive ranges yet
                "for i in 0..=100 {}", // Only '..' is supported, there are no inclusive ranges yet
            ],
        );
    }

    #[test]
    fn parse_function() {
        parse_all(
            function_definition(false),
            vec![
                "fn func_name() {}",
                "fn f(foo: pub u8, y : pub Field) -> u8 { x + a }",
                "fn f(f: pub Field, y : Field, z : comptime Field) -> u8 { x + a }",
                "fn func_name(f: Field, y : pub Field, z : pub [u8;5],) {}",
                "fn func_name(x: [Field], y : [Field;2],y : pub [Field;2], z : pub [u8;5])  {}",
            ],
        );

        parse_all_failing(
            function_definition(false),
            vec!["fn x2( f: []Field,,) {}", "fn ( f: []Field) {}", "fn ( f: []Field) {}"],
        );
    }

    #[test]
    fn parse_parenthesized_expression() {
        parse_all(atom(expression()), vec!["(0)", "(x+a)", "({(({{({(nested)})}}))})"]);
        parse_all_failing(atom(expression()), vec!["(x+a", "((x+a)", "(,)"]);
    }

    #[test]
    fn parse_tuple() {
        parse_all(tuple(expression()), vec!["()", "(x,)", "(a,b+2)", "(a,(b,c,),d,)"]);
    }

    #[test]
    fn parse_if_expr() {
        parse_all(
            if_expr(expression()),
            vec!["if x + a {  } else {  }", "if x {}", "if x {} else if y {} else {}"],
        );

        parse_all_failing(
            if_expr(expression()),
            vec!["if (x / a) + 1 {} else", "if foo then 1 else 2", "if true { 1 }else 3"],
        );
    }

    fn expr_to_lit(expr: ExpressionKind) -> Literal {
        match expr {
            ExpressionKind::Literal(literal) => literal,
            _ => unreachable!("expected a literal"),
        }
    }

    #[test]
    fn parse_int() {
        let int = parse_with(literal(), "5").unwrap();
        let hex = parse_with(literal(), "0x05").unwrap();

        match (expr_to_lit(int), expr_to_lit(hex)) {
            (Literal::Integer(int), Literal::Integer(hex)) => assert_eq!(int, hex),
            _ => unreachable!(),
        }
    }

    #[test]
    fn parse_string() {
        let expr = parse_with(literal(), r#""hello""#).unwrap();
        match expr_to_lit(expr) {
            Literal::Str(s) => assert_eq!(s, "hello"),
            _ => unreachable!(),
        };
    }

    #[test]
    fn parse_bool() {
        let expr_true = parse_with(literal(), "true").unwrap();
        let expr_false = parse_with(literal(), "false").unwrap();

        match (expr_to_lit(expr_true), expr_to_lit(expr_false)) {
            (Literal::Bool(t), Literal::Bool(f)) => {
                assert!(t);
                assert!(!f);
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn parse_module_declaration() {
        parse_with(module_declaration(), "mod foo").unwrap();
        parse_with(module_declaration(), "mod 1").unwrap_err();
    }

    #[test]
    fn parse_path() {
        let cases = vec![
            ("std", vec!["std"]),
            ("std::hash", vec!["std", "hash"]),
            ("std::hash::collections", vec!["std", "hash", "collections"]),
            ("dep::foo::bar", vec!["foo", "bar"]),
            ("crate::std::hash", vec!["std", "hash"]),
        ];

        for (src, expected_segments) in cases {
            let path: Path = parse_with(path(), src).unwrap();
            for (segment, expected) in path.segments.into_iter().zip(expected_segments) {
                assert_eq!(segment.0.contents, expected);
            }
        }

        parse_all_failing(path(), vec!["std::", "::std", "std::hash::", "foo::1"]);
    }

    #[test]
    fn parse_path_kinds() {
        let cases = vec![
            ("std", PathKind::Plain),
            ("dep::hash::collections", PathKind::Dep),
            ("crate::std::hash", PathKind::Crate),
        ];

        for (src, expected_path_kind) in cases {
            let path = parse_with(path(), src).unwrap();
            assert_eq!(path.kind, expected_path_kind)
        }

        parse_all_failing(
            path(),
            vec!["dep", "crate", "crate::std::crate", "foo::bar::crate", "foo::dep"],
        );
    }

    #[test]
    fn parse_unary() {
        parse_all(term(expression()), vec!["!hello", "-hello", "--hello", "-!hello", "!-hello"]);
        parse_all_failing(term(expression()), vec!["+hello", "/hello"]);
    }

    #[test]
    fn parse_use() {
        parse_all(
            use_statement(),
            vec!["use std::hash", "use std", "use foo::bar as hello", "use bar as bar"],
        );

        parse_all_failing(
            use_statement(),
            vec!["use std as ;", "use foobar as as;", "use hello:: as foo;"],
        );
    }

    #[test]
    fn parse_structs() {
        let cases = vec![
            "struct Foo { }",
            "struct Bar { ident: Field, }",
            "struct Baz { ident: Field, other: Field }",
        ];
        parse_all(struct_definition(), cases);

        let failing = vec!["struct {  }", "struct Foo { bar: pub Field }"];
        parse_all_failing(struct_definition(), failing);
    }

    #[test]
    fn parse_member_access() {
        let cases = vec!["a.b", "a + b.c", "foo.bar as i32"];
        parse_all(expression(), cases);
    }

    #[test]
    fn parse_constructor() {
        let cases = vec![
            "Bar { ident: 32 }",
            "Baz { other: 2 + 42, ident: foo() + 1 }",
            "Baz { other, ident: foo() + 1, foo }",
        ];
        parse_all(expression(), cases);

        // TODO: Constructor expressions with no fields are currently
        // disallowed, they conflict with block syntax in some cases. Namely:
        // if a + b {}
        // for i in 0..a { }
        // https://github.com/noir-lang/noir/issues/152
        parse_with(expression(), "Foo {}").unwrap_err();
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
        parse_all(block(expression()), cases);

        let failing = vec![
            // We disallow multiple semicolons after a statement unlike rust where it is a warning
            "{ test;; foo }",
            "{ for x in 0..1 {} foo if false {} }",
            "{ let x = 2 }",
            "{ expr1 expr2 }",
        ];
        parse_all_failing(block(expression()), failing);
    }

    #[test]
    fn statement_recovery() {
        let cases = vec![
            ("let a = 4 + 3", 0, "let a: unspecified = (4 + 3)"),
            ("let a: = 4 + 3", 1, "let a: error = (4 + 3)"),
            ("let = 4 + 3", 1, "let $error: unspecified = (4 + 3)"),
            ("let = ", 2, "let $error: unspecified = Error"),
            ("let", 3, "let $error: unspecified = Error"),
            ("foo = one two three", 1, "foo = plain::one"),
            ("constrain", 1, "constrain Error"),
            ("constrain x ==", 1, "constrain (plain::x == Error)"),
        ];

        let show_errors = |v| vecmap(v, ToString::to_string).join("\n");

        for (src, expected_errors, expected_result) in cases {
            let (opt, errors) = parse_recover(statement(expression()), src);
            let actual = opt.map(|ast| ast.to_string());
            let actual = if let Some(s) = &actual { s } else { "(none)" };

            assert_eq!((errors.len(), actual), (expected_errors, expected_result),
                "\nExpected {} error(s) and got {}:\n\n{}\n\nFrom input:   {}\nExpected AST: {}\nActual AST:   {}\n",
                expected_errors, errors.len(), show_errors(&errors), src, expected_result, actual
            );
        }
    }
}
