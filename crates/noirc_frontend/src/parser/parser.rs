use std::ops::Range;

use super::{NoirParser, ParseError, ParsedModule, Precedence, TopLevelStatement, foldl_with_span};
use crate::lexer::Lexer;
use crate::{AssignStatement, BinaryOpKind, BlockExpression, CastExpression, ConstStatement, ConstrainStatement, ForExpression, FunctionDefinition, Ident, IfExpression, ImportStatement, InfixExpression, LetStatement, Path, PathKind, PrivateStatement, UnaryOp};
use crate::token::{Attribute, Keyword, SpannedToken, Token, TokenKind};
use crate::{
    ast::{ArraySize, Expression, ExpressionKind, Statement, Type},
    FieldElementType,
};

use chumsky::prelude::*;
use noirc_errors::{Span, Spanned};

/// A Program corresponds to a single module
/// TODO: We can change this to use 'parse_recovery' and
/// return a (ParsedModule, Vec<ParseError>) instead
pub fn parse_program(program: &str) -> Result<ParsedModule, Vec<ParseError>> {
    let mut lexer = Lexer::new(program);
    let mut program = ParsedModule::with_capacity(lexer.by_ref().approx_len());
    let (tokens, lexing_errors) = lexer.lex();

    let parser = top_level_statement().repeated();
    match parser.parse(tokens) {
        Ok(statements) => {
            for statement in statements {
                match statement {
                    TopLevelStatement::Function(f) => program.push_function(f),
                    TopLevelStatement::Module(m) => program.push_module_decl(m),
                    TopLevelStatement::Import(i) => program.push_import(i),
                }
            }
            Ok(program)
        },
        Err(mut parsing_errors) => {
            let mut errors: Vec<_> = lexing_errors.into_iter().map(Into::into).collect();
            errors.append(&mut parsing_errors);
            Err(errors)
        }
    }
}

fn top_level_statement() -> impl NoirParser<TopLevelStatement> {
    choice((
        function_definition(),
        module_declaration(),
        use_statement(),
    ))
}

fn function_definition() -> impl NoirParser<TopLevelStatement> {
    attribute().or_not()
        .then_ignore(keyword(Keyword::Fn))
        .then(ident())
        .then(function_parameters().parenthesized())
        .then(function_return_type())
        .then(block())
        .map(|((((attribute, name), parameters), return_type), body)| {
            TopLevelStatement::Function(FunctionDefinition {
                span: name.0.span(),
                name,
                attribute, // XXX: Currently we only have one attribute defined. If more attributes are needed per function, we can make this a vector and make attribute definition more expressive
                parameters,
                body,
                return_type,
            }.into())
        })
}

fn function_return_type() -> impl NoirParser<Type> {
    token(Token::Arrow).ignore_then(parse_type()).or_not()
        .map(|r#type| r#type.unwrap_or(Type::Unit))
}

fn attribute() -> impl NoirParser<Attribute> {
    tokenkind(TokenKind::Attribute).map(|spanned_token| {
        match spanned_token.into_token() {
            Token::Attribute(attribute) => attribute,
            _ => unreachable!(),
        }
    })
}

fn function_parameters() -> impl NoirParser<Vec<(Ident, Type)>> {
    ident()
        .then_ignore(token(Token::Colon))
        .then(parse_type())
        .separated_by(token(Token::Comma))
        .allow_trailing()
}

fn block() -> impl NoirParser<BlockExpression> {
    statement()
        .map(|statement| {
            match statement {
                Statement::Expression(expr) => Statement::Semi(expr),
                other => other,
            }
        })
        .separated_by(token(Token::Semicolon))
        .then(token(Token::Semicolon).or_not())
        .surrounded_by(Token::LeftBrace, Token::RightBrace)
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
    token(Token::Colon).ignore_then(parse_type()).or_not()
        .map(|r#type| r#type.unwrap_or(Type::Unspecified))
}

fn module_declaration() -> impl NoirParser<TopLevelStatement> {
    keyword(Keyword::Mod).ignore_then(ident())
        .map(TopLevelStatement::Module)
}

fn use_statement() -> impl NoirParser<TopLevelStatement> {
    keyword(Keyword::Use)
        .ignore_then(path())
        .then(keyword(Keyword::As).ignore_then(ident()).or_not())
        .map(|(path, alias)| TopLevelStatement::Import(ImportStatement { path, alias }))
}

fn keyword(keyword: Keyword) -> impl NoirParser<()> {
    token(Token::Keyword(keyword))
}

fn token(expected: Token) -> impl NoirParser<()> {
    filter_map(move |span, token: SpannedToken| {
        if token.token() == &expected {
            Ok(())
        } else {
            Err(Simple::custom(span, format!("Unexpected token {}, expected {}", token.token(), expected)))
        }
    })
}

fn tokenkind(tokenkind: TokenKind) -> impl NoirParser<SpannedToken> {
    filter_map(move |span, token: SpannedToken| {
        if token.kind() == tokenkind {
            Ok(token)
        } else {
            Err(Simple::custom(span, format!("Unexpected token {}, expected {}", token.token(), tokenkind)))
        }
    })
}

fn path() -> impl NoirParser<Path> {
    let idents = || ident().separated_by(token(Token::DoubleColon));
    let make_path = |kind| move |segments| Path { segments, kind };

    choice((
        keyword(Keyword::Crate).ignore_then(idents()).map(make_path(PathKind::Crate)),
        keyword(Keyword::Dep).ignore_then(idents()).map(make_path(PathKind::Dep)),
        idents().map(make_path(PathKind::Plain)),
    ))
}

fn ident() -> impl NoirParser<Ident> {
    tokenkind(TokenKind::Ident).map(Into::into)
}

fn statement() -> impl NoirParser<Statement> {
    choice((
        constrain(),
        declaration(),
        assignment(),

        // TODO: Semi-Expr is expression ';'
        expression().map(Statement::Expression),
    ))
}

fn operator_disallowed_in_constrain(operator: BinaryOpKind) -> bool {
    [
        BinaryOpKind::And,
        BinaryOpKind::Or,
        BinaryOpKind::Divide,
        BinaryOpKind::Multiply,
    ].contains(&operator)
}

fn constrain() -> impl NoirParser<Statement> {
    keyword(Keyword::Constrain)
        .ignore_then(expression())
        .try_map(|expr, span| {
            match expr.kind.into_infix() {
                Some(infix) if infix.operator.contents == BinaryOpKind::Assign => {
                    Err(Simple::custom(span, "Cannot use '=' with a constrain statement".to_string()))
                }
                Some(infix) if operator_disallowed_in_constrain(infix.operator.contents) => {
                    let message = format!(
                        "Cannot use the {} operator in a constraint statement.",
                        infix.operator.contents.as_string()
                    );
                    Err(Simple::custom(span, message))
                }
                None => Err(Simple::custom(span, "Expected an infix expression since this is a constrain statement. You cannot assign values".to_string())),
                Some(infix) => Ok(Statement::Constrain(ConstrainStatement(infix))),
            }
        })
}

fn declaration() -> impl NoirParser<Statement> {
    fn generic_declaration<F>(key: Keyword, f: F) -> impl NoirParser<Statement>
        where F: Fn(((Ident, Type), Expression)) -> Statement
    {
        keyword(key)
            .ignore_then(ident())
            .then(optional_type_annotation())
            .then_ignore(token(Token::Assign))
            .then(expression())
            .map(f)
    }

    let let_statement = generic_declaration(Keyword::Let, |((identifier, r#type), expression)| {
        Statement::Let(LetStatement { identifier, r#type, expression })
    });

    let priv_statement = generic_declaration(Keyword::Priv, |((identifier, r#type), expression)| {
        Statement::Private(PrivateStatement { identifier, r#type, expression })
    });

    let const_statement = generic_declaration(Keyword::Const, |((identifier, r#type), expression)| {
        Statement::Const(ConstStatement { identifier, r#type, expression })
    });

    choice((let_statement, priv_statement, const_statement))
}

fn assignment() -> impl NoirParser<Statement> {
    ident()
        .then_ignore(token(Token::Assign))
        .then(expression())
        .map(|(identifier, expression)| Statement::Assign(AssignStatement {
            identifier,
            expression,
        }))
}

fn parse_type() -> impl NoirParser<Type> {
    parse_type_with_visibility(true)
}

fn parse_type_no_field_element() -> impl NoirParser<Type> {
    parse_type_with_visibility(false)
}

fn parse_type_with_visibility(parse_visibility: bool) -> impl NoirParser<Type> {
    choice((
        field_type(parse_visibility),
        int_type(parse_visibility),
        array_type(parse_visibility),
    ))
}

fn field_type(parse_visibility: bool) -> impl NoirParser<Type> {
    optional_visibility().then_ignore(keyword(Keyword::Field))
        .try_map(move |field, span| {
            let field = check_visibility(field, span, parse_visibility)?;
            Ok(Type::FieldElement(field))
        })
}

fn optional_visibility() -> impl NoirParser<Option<FieldElementType>> {
    choice((
        keyword(Keyword::Pub).map(|_| FieldElementType::Public),
        keyword(Keyword::Priv).map(|_| FieldElementType::Private),
        keyword(Keyword::Const).map(|_| FieldElementType::Constant),
    )).or_not()
}

fn check_visibility(visibility: Option<FieldElementType>, span: Range<usize>, parse_visibility: bool) -> Result<FieldElementType, ParseError> {
    match (visibility, parse_visibility) {
        (Some(visibility), true) => Ok(visibility),
        (None, _) => Ok(FieldElementType::Private),
        (Some(visibility), false) => Err(Simple::custom(span, format!("Unexpected {} found, visibility keywords aren't valid in this position", visibility))),
    }
}

fn int_type(parse_visibility: bool) -> impl NoirParser<Type> {
    optional_visibility().then(filter_map(|span, token: SpannedToken| {
        match token.into_token() {
            Token::IntType(int_type) => Ok(int_type),
            unexpected => Err(Simple::custom(span, format!("Expected an integer type, found {}", unexpected))),
        }
    })).try_map(move |(visibility, int_type), span| {
        let visibility = check_visibility(visibility, span, parse_visibility)?;
        Ok(Type::from_int_tok(visibility, &int_type))
    })
}

fn array_type(parse_visibility: bool) -> impl NoirParser<Type> {
    optional_visibility()
        .then(fixed_array_size().or_not())
        .surrounded_by(Token::LeftBracket, Token::RightBracket)
        .then(parse_type_no_field_element())
        .try_map(move |((visibility, size), element_type), span| {
            match &element_type {
                Type::Array(..) => return Err(Simple::custom(span, "Multi-dimensional arrays are currently unsupported".to_string())),
                _ => (),
            }
            let size = size.unwrap_or(ArraySize::Variable);
            let visibility = check_visibility(visibility, span, parse_visibility)?;
            Ok(Type::Array(visibility, size, Box::new(element_type)))
        })
        .boxed()
}

fn expression() -> impl NoirParser<Expression> {
    expression_with_precedence(Precedence::Lowest)
}

// NOTE: This relies on the topmost precedence level being uninhabited
fn expression_with_precedence(precedence: Precedence) -> impl NoirParser<Expression> {
    term().then(
        operator_with_precedence(precedence)
            .then(expression_with_precedence(precedence.higher()))
            .repeated()
    ).foldl(create_infix_expression).boxed()
}

fn create_infix_expression(lhs: Expression, (operator, rhs): (Spanned<BinaryOpKind>, Expression)) -> Expression {
    let span = lhs.span.merge(rhs.span);
    let kind = ExpressionKind::Infix(Box::new(InfixExpression { lhs, operator, rhs }));
    Expression { kind, span }
}

const NO_ERROR: String = String::new();

fn operator_with_precedence(precedence: Precedence) -> impl NoirParser<Spanned<BinaryOpKind>> {
    filter_map(move |span, token: SpannedToken| {
        if Precedence::token_precedence(token.token()) == Some(precedence) {
            let bin_op_kind: Option<BinaryOpKind> = token.token().into();
            Ok(Spanned::from(token.to_span(), bin_op_kind.unwrap()))
        } else {
            // This error will never actually show in user code, so best avoid
            // extra allocations entirely
            Err(Simple::custom(span, NO_ERROR))
        }
    })
}

fn term() -> impl NoirParser<Expression> {
    choice((
        if_expr(),
        for_expr(),
        array_expr(),
        not(),
        negation(),
        block().map(ExpressionKind::Block),
    )).map_with_span(to_expression)
        .or(expression().parenthesized())
        .or(value_or_cast())
        .boxed()
}

fn to_expression(kind: ExpressionKind, range: Range<usize>) -> Expression {
    Expression { kind, span: Span::new(range) }
}

fn if_expr() -> impl NoirParser<ExpressionKind> {
    keyword(Keyword::If)
        .ignore_then(expression())
        .then(block())
        .then(keyword(Keyword::Else).ignore_then(block()).or_not())
        .map(|((condition, consequence), alternative)| {
            ExpressionKind::If(Box::new(IfExpression {
                condition,
                consequence,
                alternative,
            }))
        })
}

fn for_expr() -> impl NoirParser<ExpressionKind> {
    keyword(Keyword::For)
        .ignore_then(ident())
        .then_ignore(keyword(Keyword::In))
        .then(expression())
        .then_ignore(token(Token::DoubleDot))
        .then(expression())
        .then(block())
        .map(|(((identifier, start_range), end_range), block)| {
            ExpressionKind::For(Box::new(ForExpression {
                identifier, start_range, end_range, block
            }))
        })
}

fn array_expr() -> impl NoirParser<ExpressionKind> {
    expression_list().surrounded_by(Token::LeftBracket, Token::RightBracket)
        .map(ExpressionKind::array)
}

fn expression_list() -> impl NoirParser<Vec<Expression>> {
    expression().separated_by(token(Token::Comma)).allow_trailing()
}

fn not() -> impl NoirParser<ExpressionKind> {
    token(Token::Bang).ignore_then(term())
        .map(|rhs| ExpressionKind::prefix(UnaryOp::Not, rhs))
}

fn negation() -> impl NoirParser<ExpressionKind> {
    token(Token::Minus).ignore_then(term())
        .map(|rhs| ExpressionKind::prefix(UnaryOp::Minus, rhs))
}

fn value() -> impl NoirParser<Expression> {
    choice((
        function_call(),
        array_access(),
        variable(),
        literal(),
    )).map_with_span(to_expression)
}

// This function is parses a value followed by 0 or more cast expressions.
fn value_or_cast() -> impl NoirParser<Expression> {
    let cast_rhs = keyword(Keyword::As).ignore_then(parse_type_no_field_element());

    foldl_with_span(value(), cast_rhs, |(lhs, lhs_span), (r#type, rhs_span)| {
        Expression {
            kind: ExpressionKind::Cast(Box::new(CastExpression { lhs, r#type })),
            span: lhs_span.merge(rhs_span),
        }
    })
}

fn function_call() -> impl NoirParser<ExpressionKind> {
    path().then(expression_list().surrounded_by(Token::LeftParen, Token::RightParen))
        .map(|(path, args)| ExpressionKind::function_call(path, args))
}

fn array_access() -> impl NoirParser<ExpressionKind> {
    ident().then(expression().surrounded_by(Token::LeftBracket, Token::RightBracket))
        .map(|(variable, index)| ExpressionKind::index(variable, index))
}

fn variable() -> impl NoirParser<ExpressionKind> {
    ident().map(|name| ExpressionKind::Ident(name.0.contents))
}

fn literal() -> impl NoirParser<ExpressionKind> {
    tokenkind(TokenKind::Literal).map(|token| {
        match token.into_token() {
            Token::Int(x) => ExpressionKind::integer(x),
            Token::Bool(b) => ExpressionKind::boolean(b),
            Token::Str(s) => ExpressionKind::string(s),
            unexpected => unreachable!("Non-literal {} parsed as a literal", unexpected),
        }
    })
}

fn fixed_array_size() -> impl NoirParser<ArraySize> {
    filter_map(|span, token: SpannedToken| {
        match token.into_token() {
            Token::Int(integer) => {
                if !integer.fits_in_u128() {
                    let message = "Array sizes must fit within a u128".to_string();
                    Err(Simple::custom(span, message))
                } else {
                    Ok(ArraySize::Fixed(integer.to_u128()))
                }
            }
            _ => {
                let message = "The array size is defined as [k] for fixed size or [] for variable length. k must be a literal".to_string();
                Err(Simple::custom(span, message))
            }
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
