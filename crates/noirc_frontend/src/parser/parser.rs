use super::{
    foldl_with_span, parenthesized, ExprParser, NoirParser, ParsedModule, ParserError, Precedence,
    TopLevelStatement,
};
use crate::lexer::Lexer;
use crate::token::{Attribute, Keyword, Token, TokenKind};
use crate::{
    ast::{ArraySize, Expression, ExpressionKind, Statement, Type},
    FieldElementType,
};
use crate::{
    AssignStatement, BinaryOp, BinaryOpKind, BlockExpression, CastExpression, ConstStatement,
    ConstrainStatement, ForExpression, FunctionDefinition, Ident, IfExpression, ImportStatement,
    InfixExpression, LetStatement, Path, PathKind, PrivateStatement, UnaryOp,
};

use chumsky::prelude::*;
use noirc_errors::Spanned;

/// TODO: We can leverage 'parse_recovery' and return both
/// (ParsedModule, Vec<ParseError>) instead of only one
pub fn parse_program(program: &str) -> Result<ParsedModule, Vec<ParserError>> {
    let lexer = Lexer::new(program);

    const APPROX_CHARS_PER_FUNCTION: usize = 250;
    let mut program = ParsedModule::with_capacity(lexer.approx_len() / APPROX_CHARS_PER_FUNCTION);
    let (tokens, lexing_errors) = lexer.lex();

    let parser = top_level_statement()
        .repeated()
        .then_ignore(just(Token::EOF));

    let (tree, mut parsing_errors) = parser.parse_recovery(tokens);
    match tree {
        Some(statements) => {
            for statement in statements {
                match statement {
                    TopLevelStatement::Function(f) => program.push_function(f),
                    TopLevelStatement::Module(m) => program.push_module_decl(m),
                    TopLevelStatement::Import(i) => program.push_import(i),
                }
            }
            Ok(program)
        }
        None => {
            let mut errors: Vec<_> = lexing_errors.into_iter().map(Into::into).collect();
            errors.append(&mut parsing_errors);
            Err(errors)
        }
    }
}

fn top_level_statement() -> impl NoirParser<TopLevelStatement> {
    choice((
        function_definition(),
        module_declaration().then_ignore(just(Token::Semicolon)),
        use_statement().then_ignore(just(Token::Semicolon)),
    ))
}

fn function_definition() -> impl NoirParser<TopLevelStatement> {
    attribute()
        .or_not()
        .then_ignore(keyword(Keyword::Fn))
        .then(ident())
        .then(parenthesized(function_parameters()))
        .then(function_return_type())
        .then(block(expression()))
        .map(|((((attribute, name), parameters), return_type), body)| {
            TopLevelStatement::Function(
                FunctionDefinition {
                    span: name.0.span(),
                    name,
                    attribute, // XXX: Currently we only have one attribute defined. If more attributes are needed per function, we can make this a vector and make attribute definition more expressive
                    parameters,
                    body,
                    return_type,
                }
                .into(),
            )
        })
}

fn function_return_type() -> impl NoirParser<Type> {
    just(Token::Arrow)
        .ignore_then(parse_type())
        .or_not()
        .map(|r#type| r#type.unwrap_or(Type::Unit))
}

fn attribute() -> impl NoirParser<Attribute> {
    tokenkind(TokenKind::Attribute).map(|token| match token {
        Token::Attribute(attribute) => attribute,
        _ => unreachable!(),
    })
}

fn function_parameters() -> impl NoirParser<Vec<(Ident, Type)>> {
    ident()
        .then_ignore(just(Token::Colon))
        .then(parse_type())
        .separated_by(just(Token::Comma))
        .allow_trailing()
}

fn block<P>(expr_parser: P) -> impl NoirParser<BlockExpression>
where
    P: ExprParser,
{
    statement(expr_parser)
        .map(|statement| match statement {
            Statement::Expression(expr) => Statement::Semi(expr),
            other => other,
        })
        .separated_by(just(Token::Semicolon))
        .then(just(Token::Semicolon).or_not())
        .delimited_by(Token::LeftBrace, Token::RightBrace)
        .map(|(mut statements, last_semi)| {
            if last_semi.is_none() {
                use Statement::{Expression, Semi};
                match statements.pop() {
                    Some(Semi(expr)) => statements.push(Expression(expr)),
                    Some(other) => statements.push(other),
                    None => (),
                }
            }
            BlockExpression(statements)
        })
}

fn optional_type_annotation() -> impl NoirParser<Type> {
    just(Token::Colon)
        .ignore_then(parse_type())
        .or_not()
        .map(|r#type| r#type.unwrap_or(Type::Unspecified))
}

fn module_declaration() -> impl NoirParser<TopLevelStatement> {
    keyword(Keyword::Mod)
        .ignore_then(ident())
        .map(TopLevelStatement::Module)
}

fn use_statement() -> impl NoirParser<TopLevelStatement> {
    keyword(Keyword::Use)
        .ignore_then(path())
        .then(keyword(Keyword::As).ignore_then(ident()).or_not())
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
    let prefix = |key| keyword(key).ignore_then(just(Token::DoubleColon));
    let idents = || ident().separated_by(just(Token::DoubleColon));
    let make_path = |kind| move |segments| Path { segments, kind };

    choice((
        prefix(Keyword::Crate)
            .ignore_then(idents())
            .map(make_path(PathKind::Crate)),
        prefix(Keyword::Dep)
            .ignore_then(idents())
            .map(make_path(PathKind::Dep)),
        idents().map(make_path(PathKind::Plain)),
    ))
}

fn ident() -> impl NoirParser<Ident> {
    tokenkind(TokenKind::Ident).map_with_span(Ident::new)
}

fn statement<P>(expr_parser: P) -> impl NoirParser<Statement>
where
    P: ExprParser,
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

fn constrain<P>(expr_parser: P) -> impl NoirParser<Statement>
where
    P: ExprParser,
{
    keyword(Keyword::Constrain)
        .ignore_then(expr_parser)
        .try_map(|expr, span| match expr.kind.into_infix() {
            Some(infix) if operator_disallowed_in_constrain(infix.operator.contents) => {
                Err(ParserError::invalid_constrain_operator(infix.operator))
            }
            None => Err(ParserError::with_reason(
                "Only an infix expression can follow the constrain keyword".to_string(),
                span,
            )),
            Some(infix) => Ok(Statement::Constrain(ConstrainStatement(infix))),
        })
}

fn declaration<P>(expr_parser: P) -> impl NoirParser<Statement>
where
    P: ExprParser,
{
    let let_statement = generic_declaration(
        Keyword::Let,
        expr_parser.clone(),
        |((identifier, r#type), expression)| {
            Statement::Let(LetStatement {
                identifier,
                r#type,
                expression,
            })
        },
    );

    let priv_statement = generic_declaration(
        Keyword::Priv,
        expr_parser.clone(),
        |((identifier, r#type), expression)| {
            Statement::Private(PrivateStatement {
                identifier,
                r#type,
                expression,
            })
        },
    );

    let const_statement = generic_declaration(
        Keyword::Const,
        expr_parser,
        |((identifier, r#type), expression)| {
            Statement::Const(ConstStatement {
                identifier,
                r#type,
                expression,
            })
        },
    );

    choice((let_statement, priv_statement, const_statement))
}

fn generic_declaration<F, P>(key: Keyword, expr_parser: P, f: F) -> impl NoirParser<Statement>
where
    F: Fn(((Ident, Type), Expression)) -> Statement,
    P: ExprParser,
{
    keyword(key)
        .ignore_then(ident())
        .then(optional_type_annotation())
        .then_ignore(just(Token::Assign))
        .then(expr_parser)
        .map(f)
}

fn assignment<P>(expr_parser: P) -> impl NoirParser<Statement>
where
    P: ExprParser,
{
    ident()
        .then_ignore(just(Token::Assign))
        .then(expr_parser)
        .map(|(identifier, expression)| {
            Statement::Assign(AssignStatement {
                identifier,
                expression,
            })
        })
}

fn parse_type() -> impl NoirParser<Type> {
    choice((
        field_type(optional_visibility()),
        int_type(optional_visibility()),
        array_type(optional_visibility(), parse_type_no_field_element()),
    ))
}

fn parse_type_no_field_element() -> impl NoirParser<Type> {
    // NOTE: Technically since we disallow multidimensional arrays our type parser
    // does not strictly need to be recursive - we could manually unroll it by
    // only parsing an integer or field type as our array elements. If/when Noir's
    // types become truly recursive though this will be necessary
    recursive(|type_parser| {
        choice((
            field_type(no_visibility()),
            int_type(no_visibility()),
            array_type(no_visibility(), type_parser),
        ))
    })
    .boxed()
}

// Parse nothing, just return a FieldElementType::Private
fn no_visibility() -> impl NoirParser<FieldElementType> {
    just([]).or_not().map(|_| FieldElementType::Private)
}

fn optional_visibility() -> impl NoirParser<FieldElementType> {
    choice((
        keyword(Keyword::Pub).map(|_| FieldElementType::Public),
        keyword(Keyword::Priv).map(|_| FieldElementType::Private),
        keyword(Keyword::Const).map(|_| FieldElementType::Constant),
    ))
    .or_not()
    .map(|opt| opt.unwrap_or(FieldElementType::Private))
}

fn field_type<P>(visibility_parser: P) -> impl NoirParser<Type>
where
    P: NoirParser<FieldElementType>,
{
    visibility_parser
        .then_ignore(keyword(Keyword::Field))
        .map(Type::FieldElement)
}

fn int_type<P>(visibility_parser: P) -> impl NoirParser<Type>
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
        .map(|(visibility, int_type)| Type::from_int_tok(visibility, &int_type))
}

fn array_type<V, T>(visibility_parser: V, type_parser: T) -> impl NoirParser<Type>
where
    V: NoirParser<FieldElementType>,
    T: NoirParser<Type>,
{
    visibility_parser
        .then_ignore(just(Token::LeftBracket))
        .then(fixed_array_size().or_not())
        .then_ignore(just(Token::RightBracket))
        .then(type_parser)
        .try_map(|((visibility, size), element_type), span| {
            if let Type::Array(..) = &element_type {
                return Err(ParserError::with_reason(
                    "Multi-dimensional arrays are currently unsupported".to_string(),
                    span,
                ));
            }
            let size = size.unwrap_or(ArraySize::Variable);
            Ok(Type::Array(visibility, size, Box::new(element_type)))
        })
}

fn expression() -> impl ExprParser {
    recursive(|expr_parser| expression_with_precedence(Precedence::Lowest, expr_parser))
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
        term(expr_parser).boxed()
    } else {
        expression_with_precedence(precedence.higher(), expr_parser.clone())
            .then(
                operator_with_precedence(precedence)
                    .then(expression_with_precedence(precedence.higher(), expr_parser))
                    .repeated(),
            )
            .foldl(create_infix_expression)
            .boxed()
    }
}

fn create_infix_expression(lhs: Expression, (operator, rhs): (BinaryOp, Expression)) -> Expression {
    let span = lhs.span.merge(rhs.span);
    let is_comparator = operator.contents.is_comparator();
    let infix = Box::new(InfixExpression { lhs, operator, rhs });

    if is_comparator {
        Expression {
            span,
            kind: ExpressionKind::Predicate(infix),
        }
    } else {
        Expression {
            span,
            kind: ExpressionKind::Infix(infix),
        }
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
        .or(parenthesized(expr_parser.clone()))
        .or(value_or_cast(expr_parser))
    })
}

fn if_expr<P>(expr_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
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

fn for_expr<P>(expr_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
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
        .delimited_by(Token::LeftBracket, Token::RightBracket)
        .map(ExpressionKind::array)
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
        array_access(expr_parser),
        variable(),
        literal(),
    ))
    .map_with_span(Expression::new)
}

// This function is parses a value followed by 0 or more cast expressions.
fn value_or_cast<P>(expr_parser: P) -> impl NoirParser<Expression>
where
    P: ExprParser,
{
    let cast_rhs = keyword(Keyword::As).ignore_then(parse_type_no_field_element());

    foldl_with_span(
        value(expr_parser),
        cast_rhs,
        |(lhs, lhs_span), (r#type, rhs_span)| Expression {
            kind: ExpressionKind::Cast(Box::new(CastExpression { lhs, r#type })),
            span: lhs_span.merge(rhs_span),
        },
    )
}

fn function_call<P>(expr_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    path()
        .then(expression_list(expr_parser).delimited_by(Token::LeftParen, Token::RightParen))
        .map(|(path, args)| ExpressionKind::function_call(path, args))
}

fn array_access<P>(expr_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    ident()
        .then(expr_parser.delimited_by(Token::LeftBracket, Token::RightBracket))
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
    use super::*;

    #[test]
    fn regression_skip_comment() {
        const COMMENT_BETWEEN_FIELD: &str = r#"
            fn main(
                // This comment should be skipped
                x : Field,
                // And this one
                y : Field,
            ) {

            }
        "#;
        const COMMENT_BETWEEN_CALL: &str = r#"
            fn main(x : Field, y : Field,) {
                foo::bar(
                    // Comment for x argument
                    x,
                    // Comment for y argument
                    y
                )
            }
        "#;
        parse_program(COMMENT_BETWEEN_FIELD).unwrap();
        parse_program(COMMENT_BETWEEN_CALL).unwrap();
    }
}
