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
    AssignStatement, BinaryOp, BinaryOpKind, BlockExpression, CastExpression, ConstrainStatement,
    ForExpression, FunctionDefinition, Ident, IfExpression, ImportStatement, InfixExpression, Path,
    PathKind, UnaryOp,
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
        .then(parenthesized(function_parameters(), |_| vec![]))
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
    use Token::*;
    statement(expr_parser)
        .separated_by(just(Semicolon))
        .then(just(Semicolon).or_not())
        .delimited_by(LeftBrace, RightBrace)
        .recover_with(nested_delimiters(
            LeftBrace,
            RightBrace,
            [(LeftParen, RightParen), (LeftBracket, RightBracket)],
            |_| (vec![], None),
        ))
        .map(into_block)
}

fn into_block(
    (mut statements, last_semicolon): (Vec<Statement>, Option<Token>),
) -> BlockExpression {
    if last_semicolon.is_some() && !statements.is_empty() {
        let last = match statements.pop().unwrap() {
            Statement::Expression(expr) => Statement::Semi(expr),
            other => other,
        };
        statements.push(last);
    }
    BlockExpression(statements)
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
    let idents = || ident().separated_by(just(Token::DoubleColon)).at_least(1);
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
    .labelled("statement")
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
    let let_statement = generic_declaration(Keyword::Let, expr_parser.clone(), Statement::new_let);
    let priv_statement =
        generic_declaration(Keyword::Priv, expr_parser.clone(), Statement::new_priv);
    let const_statement = generic_declaration(Keyword::Const, expr_parser, Statement::new_const);

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
        array_access(expr_parser.clone()),
        variable(),
        literal(),
    ))
    .map_with_span(Expression::new)
    .or(parenthesized(expr_parser, |span| {
        Expression::new(ExpressionKind::Block(BlockExpression(vec![])), span)
    }))
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
        .then(parenthesized(expression_list(expr_parser), |_| vec![]))
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

    fn parse_with<P, T>(parser: P, program: &str) -> Result<T, Vec<ParserError>>
    where
        P: NoirParser<T>,
    {
        let lexer = Lexer::new(program);
        let (tokens, lexer_errors) = lexer.lex();
        if !lexer_errors.is_empty() {
            return Err(lexer_errors.into_iter().map(Into::into).collect());
        }
        parser.then_ignore(just(Token::EOF)).parse(tokens)
    }

    fn parse_all<P, T>(parser: P, programs: Vec<&str>) -> Vec<T>
    where
        P: NoirParser<T>,
    {
        programs
            .into_iter()
            .map(move |program| {
                let message = format!("Failed to parse:\n{}", program);
                parse_with(&parser, program).expect(&message)
            })
            .collect()
    }

    fn parse_all_failing<P, T>(parser: P, programs: Vec<&str>) -> Vec<ParserError>
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
            function_definition(),
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

    use crate::{ArrayLiteral, Literal};

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

            // All array types are unknown at parse time
            // This makes parsing simpler. The type checker
            // needs to iterate the whole array to ensure homogeneity
            // so there is no advantage to deducing the type here.
            assert_eq!(arr_lit.r#type, Type::Unknown);
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
            function_definition(),
            vec![
                "fn func_name() {}",
                "fn f(foo: pub u8, y : pub Field) -> u8 { x + a }",
                "fn f(f: pub Field, y : Field, z : const Field) -> u8 { x + a }",
                "fn func_name(f: Field, y : pub Field, z : pub [5]u8,) {}",
                "fn func_name(x: []Field, y : [2]Field,y : pub [2]Field, z : pub [5]u8)  {}",
            ],
        );

        parse_all_failing(
            function_definition(),
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
}
