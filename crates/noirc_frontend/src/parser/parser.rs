use super::{
    foldl_with_span, parenthesized, then_commit, then_commit_ignore, ExprParser, NoirParser,
    ParsedModule, ParserError, Precedence, TopLevelStatement,
};
use crate::lexer::Lexer;
use crate::parser::{force, ignore_then_commit};
use crate::token::{Attribute, Keyword, Token, TokenKind};
use crate::util::vecmap;
use crate::{
    ast::{ArraySize, Expression, ExpressionKind, Statement, UnresolvedType},
    FieldElementType,
};
use crate::{
    AssignStatement, BinaryOp, BinaryOpKind, BlockExpression, ConstrainStatement, ForExpression,
    FunctionDefinition, Ident, IfExpression, ImportStatement, InfixExpression, NoirFunction,
    NoirImpl, NoirStruct, Path, PathKind, UnaryOp,
};

use chumsky::prelude::*;
use noirc_errors::{CustomDiagnostic, DiagnosableError, Span, Spanned};

pub fn parse_program(source_program: &str) -> (ParsedModule, Vec<CustomDiagnostic>) {
    let lexer = Lexer::new(source_program);
    let (tokens, lexing_errors) = lexer.lex();
    let mut errors = vecmap(&lexing_errors, DiagnosableError::to_diagnostic);

    let (module, parsing_errors) = program().parse_recovery(tokens);
    errors.extend(parsing_errors.iter().map(DiagnosableError::to_diagnostic));

    (module.unwrap(), errors)
}

fn program() -> impl NoirParser<ParsedModule> {
    empty()
        .map(move |_| ParsedModule::default())
        .then(top_level_statement().repeated())
        .then_ignore(force(just(Token::EOF)))
        .foldl(|mut program, statement| {
            match statement {
                TopLevelStatement::Function(f) => program.push_function(f),
                TopLevelStatement::Module(m) => program.push_module_decl(m),
                TopLevelStatement::Import(i) => program.push_import(i),
                TopLevelStatement::Struct(s) => program.push_type(s),
                TopLevelStatement::Impl(i) => program.push_impl(i),
            }
            program
        })
}

fn top_level_statement() -> impl NoirParser<TopLevelStatement> {
    choice((
        function_definition(false).map(TopLevelStatement::Function),
        struct_definition(),
        implementation(),
        module_declaration().then_ignore(force(just(Token::Semicolon))),
        use_statement().then_ignore(force(just(Token::Semicolon))),
    ))
}

fn function_definition(allow_self: bool) -> impl NoirParser<NoirFunction> {
    attribute()
        .or_not()
        .then_ignore(keyword(Keyword::Fn))
        .then(ident())
        .then(parenthesized(function_parameters(allow_self)))
        .then(function_return_type())
        .then(block(expression()))
        .map(|((((attribute, name), parameters), return_type), body)| {
            FunctionDefinition {
                span: name.0.span(),
                name,
                attribute, // XXX: Currently we only have one attribute defined. If more attributes are needed per function, we can make this a vector and make attribute definition more expressive
                parameters,
                body,
                return_type,
            }
            .into()
        })
}

fn struct_definition() -> impl NoirParser<TopLevelStatement> {
    keyword(Keyword::Struct)
        .ignore_then(ident())
        .then(struct_fields().delimited_by(just(Token::LeftBrace), just(Token::RightBrace)))
        .map_with_span(|(name, fields), span| {
            TopLevelStatement::Struct(NoirStruct { name, fields, span })
        })
}

fn function_return_type() -> impl NoirParser<UnresolvedType> {
    just(Token::Arrow)
        .ignore_then(parse_type())
        .or_not()
        .map(|r#type| r#type.unwrap_or(UnresolvedType::Unit))
}

fn attribute() -> impl NoirParser<Attribute> {
    tokenkind(TokenKind::Attribute).map(|token| match token {
        Token::Attribute(attribute) => attribute,
        _ => unreachable!(),
    })
}

fn struct_fields() -> impl NoirParser<Vec<(Ident, UnresolvedType)>> {
    parameters(
        false,
        parse_type_with_visibility(optional_pri_or_const(), parse_type_no_field_element()),
    )
}

fn function_parameters(allow_self: bool) -> impl NoirParser<Vec<(Ident, UnresolvedType)>> {
    parameters(allow_self, parse_type())
}

fn parameters<P>(allow_self: bool, type_parser: P) -> impl NoirParser<Vec<(Ident, UnresolvedType)>>
where
    P: NoirParser<UnresolvedType>,
{
    let full_parameter = ident().then_ignore(just(Token::Colon)).then(type_parser);

    let self_parameter = if allow_self {
        self_parameter().boxed()
    } else {
        nothing().boxed()
    };

    full_parameter
        .or(self_parameter)
        .separated_by(just(Token::Comma))
        .allow_trailing()
}

fn nothing<T>() -> impl NoirParser<T> {
    one_of([]).map(|_| unreachable!())
}

fn self_parameter() -> impl NoirParser<(Ident, UnresolvedType)> {
    filter_map(move |span, found: Token| match found {
        Token::Ident(ref word) if word == "self" => {
            let ident = Ident::new(found, span);
            let path = Path::from_single("Self".to_owned(), span);
            Ok((
                ident,
                UnresolvedType::Struct(FieldElementType::Private, path),
            ))
        }
        _ => Err(ParserError::expected_label(
            "parameter".to_owned(),
            found,
            span,
        )),
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

fn block<'a, P>(expr_parser: P) -> impl NoirParser<BlockExpression> + 'a
where
    P: ExprParser + 'a,
{
    use Token::*;
    statement(expr_parser)
        .then(just(Semicolon).or_not().map_with_span(|s, span| (s, span)))
        .repeated()
        .validate(check_statements_require_semicolon)
        .delimited_by(just(LeftBrace), just(RightBrace))
        .recover_with(nested_delimiters(
            LeftBrace,
            RightBrace,
            [(LeftParen, RightParen), (LeftBracket, RightBracket)],
            |_| vec![],
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

fn optional_type_annotation() -> impl NoirParser<UnresolvedType> {
    ignore_then_commit(just(Token::Colon), parse_type())
        .or_not()
        .map(|r#type| r#type.unwrap_or(UnresolvedType::Unspecified))
}

fn module_declaration() -> impl NoirParser<TopLevelStatement> {
    keyword(Keyword::Mod)
        .ignore_then(ident())
        .map(TopLevelStatement::Module)
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
            Err(ParserError::expected_label(
                tokenkind.to_string(),
                found,
                span,
            ))
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
    tokenkind(TokenKind::Ident).map_with_span(Ident::new)
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

fn operator_disallowed_in_constrain(operator: BinaryOpKind) -> bool {
    [
        BinaryOpKind::And,
        BinaryOpKind::Subtract,
        BinaryOpKind::Divide,
        BinaryOpKind::Multiply,
        BinaryOpKind::Or,
        BinaryOpKind::Assign,
    ]
    .contains(&operator)
}

fn constrain<'a, P>(expr_parser: P) -> impl NoirParser<Statement> + 'a
where
    P: ExprParser + 'a,
{
    ignore_then_commit(
        keyword(Keyword::Constrain).labelled("statement"),
        expr_parser,
    )
    .validate(|expr, span, emit| match expr.kind.into_infix() {
        Some(infix) if operator_disallowed_in_constrain(infix.operator.contents) => {
            emit(ParserError::invalid_constrain_operator(infix.operator));
            Statement::Error
        }
        None => {
            emit(ParserError::with_reason(
                "Only an infix expression can follow the constrain keyword".to_string(),
                span,
            ));
            Statement::Error
        }
        Some(infix) => Statement::Constrain(ConstrainStatement(infix)),
    })
}

fn declaration<'a, P>(expr_parser: P) -> impl NoirParser<Statement> + 'a
where
    P: ExprParser + 'a,
{
    let let_statement = generic_declaration(Keyword::Let, expr_parser.clone(), Statement::new_let);
    let priv_statement =
        generic_declaration(Keyword::Priv, expr_parser.clone(), Statement::new_priv);
    let const_statement = generic_declaration(Keyword::Const, expr_parser, Statement::new_const);

    choice((let_statement, priv_statement, const_statement))
}

fn generic_declaration<'a, F, P>(
    key: Keyword,
    expr_parser: P,
    f: F,
) -> impl NoirParser<Statement> + 'a
where
    F: 'a + Clone + Fn(((Ident, UnresolvedType), Expression)) -> Statement,
    P: ExprParser + 'a,
{
    let p = ignore_then_commit(keyword(key).labelled("statement"), ident());
    let p = p.then(optional_type_annotation());
    let p = then_commit_ignore(p, just(Token::Assign));
    let p = then_commit(p, expr_parser);

    p.map(f)
}

fn assignment<'a, P>(expr_parser: P) -> impl NoirParser<Statement> + 'a
where
    P: ExprParser + 'a,
{
    let failable = ident()
        .then_ignore(just(Token::Assign))
        .labelled("statement");
    then_commit(failable, expr_parser).map(|(identifier, expression)| {
        Statement::Assign(AssignStatement {
            identifier,
            expression,
        })
    })
}

fn parse_type() -> impl NoirParser<UnresolvedType> {
    parse_type_with_visibility(optional_visibility(), parse_type_no_field_element())
}

fn parse_type_no_field_element() -> impl NoirParser<UnresolvedType> {
    // NOTE: Technically since we disallow multidimensional arrays our type parser
    // does not strictly need to be recursive - we could manually unroll it by
    // only parsing an integer or field type as our array elements. If/when Noir's
    // types become truly recursive though this will be necessary
    recursive(|type_parser| parse_type_with_visibility(no_visibility(), type_parser))
}

fn parse_type_with_visibility<V, T>(
    visibility_parser: V,
    recursive_type_parser: T,
) -> impl NoirParser<UnresolvedType>
where
    V: NoirParser<FieldElementType>,
    T: NoirParser<UnresolvedType>,
{
    choice((
        field_type(visibility_parser.clone()),
        int_type(visibility_parser.clone()),
        struct_type(visibility_parser.clone()),
        array_type(visibility_parser, recursive_type_parser),
    ))
}

// Parse nothing, just return a FieldElementType::Private
fn no_visibility() -> impl NoirParser<FieldElementType> {
    just([]).or_not().map(|_| FieldElementType::Private)
}

// Returns a parser that parses any FieldElementType that satisfies
// the given predicate
fn visibility(field: FieldElementType) -> impl NoirParser<FieldElementType> {
    keyword(field.as_keyword()).map(move |_| field)
}

fn optional_visibility() -> impl NoirParser<FieldElementType> {
    choice((
        visibility(FieldElementType::Public),
        visibility(FieldElementType::Private),
        visibility(FieldElementType::Constant),
        no_visibility(),
    ))
}

// This is primarily for struct fields which cannot be public
fn optional_pri_or_const() -> impl NoirParser<FieldElementType> {
    choice((
        visibility(FieldElementType::Private),
        visibility(FieldElementType::Constant),
        no_visibility(),
    ))
}

fn field_type<P>(visibility_parser: P) -> impl NoirParser<UnresolvedType>
where
    P: NoirParser<FieldElementType>,
{
    visibility_parser
        .then_ignore(keyword(Keyword::Field))
        .map(UnresolvedType::FieldElement)
}

fn int_type<P>(visibility_parser: P) -> impl NoirParser<UnresolvedType>
where
    P: NoirParser<FieldElementType>,
{
    visibility_parser
        .then(filter_map(|span, token: Token| match token {
            Token::IntType(int_type) => Ok(int_type),
            unexpected => Err(ParserError::expected_label(
                "integer type".to_string(),
                unexpected,
                span,
            )),
        }))
        .map(|(visibility, int_type)| UnresolvedType::from_int_tok(visibility, &int_type))
}

fn struct_type<P>(visibility_parser: P) -> impl NoirParser<UnresolvedType>
where
    P: NoirParser<FieldElementType>,
{
    visibility_parser
        .then(path())
        .map(|(visibility, path)| UnresolvedType::Struct(visibility, path))
}

fn array_type<V, T>(visibility_parser: V, type_parser: T) -> impl NoirParser<UnresolvedType>
where
    V: NoirParser<FieldElementType>,
    T: NoirParser<UnresolvedType>,
{
    visibility_parser
        .then_ignore(just(Token::LeftBracket))
        .then(fixed_array_size().or_not())
        .then_ignore(just(Token::RightBracket))
        .then(type_parser)
        .try_map(|((visibility, size), element_type), span| {
            if let UnresolvedType::Array(..) = &element_type {
                return Err(ParserError::with_reason(
                    "Multi-dimensional arrays are currently unsupported".to_string(),
                    span,
                ));
            }
            let size = size.unwrap_or(ArraySize::Variable);
            Ok(UnresolvedType::Array(
                visibility,
                size,
                Box::new(element_type),
            ))
        })
}

fn expression() -> impl ExprParser {
    recursive(|expr| expression_with_precedence(Precedence::Lowest, expr)).labelled("expression")
}

// An expression is a single term followed by 0 or more (OP subexpr)*
// where OP is an operator at the given precedence level and subexpr
// is an expression at the current precedence level plus one.
fn expression_with_precedence<'a, P>(
    precedence: Precedence,
    expr_parser: P,
) -> impl NoirParser<Expression> + 'a
where
    P: ExprParser + 'a,
{
    if precedence == Precedence::Highest {
        term(expr_parser).boxed().labelled("term")
    } else {
        expression_with_precedence(precedence.higher(), expr_parser.clone())
            .then(
                then_commit(
                    operator_with_precedence(precedence),
                    expression_with_precedence(precedence.higher(), expr_parser),
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

    Expression {
        span,
        kind: ExpressionKind::Infix(infix),
    }
}

fn operator_with_precedence(precedence: Precedence) -> impl NoirParser<Spanned<BinaryOpKind>> {
    filter_map(move |span, token: Token| {
        if Precedence::token_precedence(&token) == Some(precedence) {
            let bin_op_kind: Option<BinaryOpKind> = (&token).into();
            Ok(Spanned::from(span, bin_op_kind.unwrap()))
        } else {
            Err(ParserError::expected_label(
                "binary operator".to_string(),
                token,
                span,
            ))
        }
    })
}

fn term<'a, P>(expr_parser: P) -> impl NoirParser<Expression> + 'a
where
    P: ExprParser + 'a,
{
    recursive(move |term_parser| {
        choice((
            if_expr(expr_parser.clone()),
            for_expr(expr_parser.clone()),
            array_expr(expr_parser.clone()),
            not(term_parser.clone()),
            negation(term_parser),
            block(expr_parser.clone()).map(ExpressionKind::Block),
        ))
        .map_with_span(Expression::new)
        .or(value_or_cast(expr_parser))
    })
}

fn if_expr<'a, P>(expr_parser: P) -> impl NoirParser<ExpressionKind> + 'a
where
    P: ExprParser + 'a,
{
    keyword(Keyword::If)
        .ignore_then(expr_parser.clone())
        .then(block(expr_parser.clone()))
        .then(
            keyword(Keyword::Else)
                .ignore_then(block(expr_parser))
                .or_not(),
        )
        .map(|((condition, consequence), alternative)| {
            ExpressionKind::If(Box::new(IfExpression {
                condition,
                consequence,
                alternative,
            }))
        })
}

fn for_expr<'a, P>(expr_parser: P) -> impl NoirParser<ExpressionKind> + 'a
where
    P: ExprParser + 'a,
{
    keyword(Keyword::For)
        .ignore_then(ident())
        .then_ignore(keyword(Keyword::In))
        .then(expr_parser.clone())
        .then_ignore(just(Token::DoubleDot))
        .then(expr_parser.clone())
        .then(block(expr_parser))
        .map(|(((identifier, start_range), end_range), block)| {
            ExpressionKind::For(Box::new(ForExpression {
                identifier,
                start_range,
                end_range,
                block,
            }))
        })
}

fn array_expr<P>(expr_parser: P) -> impl NoirParser<ExpressionKind>
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

fn expression_list<P>(expr_parser: P) -> impl NoirParser<Vec<Expression>>
where
    P: ExprParser,
{
    expr_parser
        .separated_by(just(Token::Comma))
        .allow_trailing()
}

fn not<P>(term_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    just(Token::Bang)
        .ignore_then(term_parser)
        .map(|rhs| ExpressionKind::prefix(UnaryOp::Not, rhs))
}

fn negation<P>(term_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    just(Token::Minus)
        .ignore_then(term_parser)
        .map(|rhs| ExpressionKind::prefix(UnaryOp::Minus, rhs))
}

fn value<P>(expr_parser: P) -> impl NoirParser<Expression>
where
    P: ExprParser,
{
    choice((
        function_call(expr_parser.clone()),
        array_access(expr_parser.clone()),
        constructor(expr_parser.clone()),
        variable(),
        literal(),
    ))
    .map_with_span(Expression::new)
    .or(parenthesized(expr_parser))
    .labelled("value")
}

// Parses a value followed by 0 or more member accesses or method calls
fn member_access<P>(expr_parser: P) -> impl NoirParser<Expression>
where
    P: ExprParser,
{
    let rhs = just(Token::Dot)
        .ignore_then(ident())
        .then(parenthesized(expression_list(expr_parser.clone())).or_not())
        .labelled("field access");

    foldl_with_span(
        value(expr_parser),
        rhs,
        Expression::member_access_or_method_call,
    )
}

// Parses a member_access followed by 0 or more cast expressions
fn value_or_cast<P>(expr_parser: P) -> impl NoirParser<Expression>
where
    P: ExprParser,
{
    let cast_rhs = keyword(Keyword::As)
        .ignore_then(parse_type_no_field_element())
        .labelled("cast");

    foldl_with_span(member_access(expr_parser), cast_rhs, Expression::cast)
}

fn function_call<P>(expr_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    path()
        .then(parenthesized(expression_list(expr_parser)))
        .map(ExpressionKind::function_call)
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

fn array_access<P>(expr_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    ident()
        .then(expr_parser.delimited_by(just(Token::LeftBracket), just(Token::RightBracket)))
        .map(|(variable, index)| ExpressionKind::index(variable, index))
}

fn variable() -> impl NoirParser<ExpressionKind> {
    ident().map(|name| ExpressionKind::Ident(name.0.contents))
}

fn literal() -> impl NoirParser<ExpressionKind> {
    tokenkind(TokenKind::Literal).map(|token| match token {
        Token::Int(x) => ExpressionKind::integer(x),
        Token::Bool(b) => ExpressionKind::boolean(b),
        Token::Str(s) => ExpressionKind::string(s),
        unexpected => unreachable!("Non-literal {} parsed as a literal", unexpected),
    })
}

fn fixed_array_size() -> impl NoirParser<ArraySize> {
    filter_map(|span, token: Token| match token {
        Token::Int(integer) => {
            if !integer.fits_in_u128() {
                let message = "Array sizes must fit within a u128".to_string();
                Err(ParserError::with_reason(message, span))
            } else {
                Ok(ArraySize::Fixed(integer.to_u128()))
            }
        }
        _ => {
            let message = "The array size is defined as [k] for fixed size or [] for variable length. k must be a literal".to_string();
            Err(ParserError::with_reason(message, span))
        }
    })
}

#[cfg(test)]
mod test {
    use noirc_errors::{CustomDiagnostic, DiagnosableError};

    use super::*;
    use crate::{ArrayLiteral, Literal, UnresolvedType};

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
        let (opt, errs) = parser
            .then_ignore(force(just(Token::EOF)))
            .parse_recovery(tokens);

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
        ];
        parse_all(function_call(expression()), valid);
    }

    #[test]
    fn parse_cast() {
        parse_all(
            value_or_cast(expression()),
            vec!["x as u8", "0 as Field", "(x + 3) as [8]Field"],
        );
        parse_all_failing(value_or_cast(expression()), vec!["x as pub u8"]);
    }

    #[test]
    fn parse_array_index() {
        let valid = vec!["x[9]", "y[x+a]", " foo [foo+5]", "baz[bar]"];
        parse_all(array_access(expression()), valid);
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

    /// This is the standard way to declare an array
    #[test]
    fn parse_array() {
        let valid = vec![
            "[0, 1, 2,3, 4]",
            "[0,1,2,3,4,]", // Trailing commas are valid syntax
        ];

        for expr in parse_all(array_expr(expression()), valid) {
            let arr_lit = expr_to_array(expr);
            assert_eq!(arr_lit.length, 5);
        }

        parse_all_failing(
            array_expr(expression()),
            vec!["0,1,2,3,4]", "[[0,1,2,3,4]", "[0,1,2,,]", "[0,1,2,3,4"],
        );
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
            BinaryOpKind::Assign,
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
        parse_all(
            declaration(expression()),
            vec!["let x = y", "let x : u8 = y"],
        );
    }
    #[test]
    fn parse_priv() {
        parse_all(
            declaration(expression()),
            vec!["priv x = y", "priv x : pub Field = y"],
        );
    }

    #[test]
    fn parse_invalid_pub() {
        // pub cannot be used to declare a statement
        parse_all_failing(
            statement(expression()),
            vec!["pub x = y", "pub x : pub Field = y"],
        );
    }

    #[test]
    fn parse_const() {
        // XXX: We have `Constant` because we may allow constants to
        // be casted to integers. Maybe rename this to `Field` instead
        parse_all(
            declaration(expression()),
            vec!["const x = y", "const x : const Field = y"],
        );
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
                "fn f(f: pub Field, y : Field, z : const Field) -> u8 { x + a }",
                "fn func_name(f: Field, y : pub Field, z : pub [5]u8,) {}",
                "fn func_name(x: []Field, y : [2]Field,y : pub [2]Field, z : pub [5]u8)  {}",
            ],
        );

        parse_all_failing(
            function_definition(false),
            vec![
                "fn x2( f: []Field,,) {}",
                "fn ( f: []Field) {}",
                "fn ( f: []Field) {}",
            ],
        );
    }

    #[test]
    fn parse_parenthesized_expression() {
        parse_all(
            value(expression()),
            vec!["(0)", "(x+a)", "({(({{({(nested)})}}))})"],
        );

        parse_all_failing(value(expression()), vec!["(x+a", "((x+a)", "()"]);
    }

    #[test]
    fn parse_if_expr() {
        parse_all(
            if_expr(expression()),
            vec!["if x + a {  } else {  }", "if x {}"],
        );

        parse_all_failing(
            if_expr(expression()),
            vec![
                "if (x / a) + 1 {} else",
                "if foo then 1 else 2",
                "if true { 1 }else 3",
            ],
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
            vec![
                "dep",
                "crate",
                "crate::std::crate",
                "foo::bar::crate",
                "foo::dep",
            ],
        );
    }

    #[test]
    fn parse_unary() {
        parse_all(
            term(expression()),
            vec!["!hello", "-hello", "--hello", "-!hello", "!-hello"],
        );
        parse_all_failing(term(expression()), vec!["+hello", "/hello"]);
    }

    #[test]
    fn parse_use() {
        parse_all(
            use_statement(),
            vec![
                "use std::hash",
                "use std",
                "use foo::bar as hello",
                "use bar as bar",
            ],
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
            ("priv = ", 2, "priv $error: unspecified = Error"),
            ("foo = one two three", 1, "foo = one"),
            ("constrain", 2, "Error"), // We don't recover 'constrain Error' since constrain needs a binary operator
            ("constrain x ==", 2, "constrain (x == Error)"), // This gives a duplicate 'end of input' error currently
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
