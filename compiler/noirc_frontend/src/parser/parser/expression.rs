use noirc_errors::Span;

use crate::{
    ast::{
        ArrayLiteral, BlockExpression, CallExpression, CastExpression, ConstructorExpression,
        Expression, ExpressionKind, Ident, IfExpression, IndexExpression, Literal,
        MemberAccessExpression, MethodCallExpression, Path, PrefixExpression, UnaryOp,
        UnresolvedType,
    },
    parser::ParserErrorReason,
    token::{Keyword, Token, TokenKind},
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_expression_or_error(&mut self) -> Expression {
        self.parse_expression_or_error_impl(true) // allow constructors
    }

    pub(crate) fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_expression_impl(true) // allow constructors
    }

    /// When parsing `if` conditions we don't allow constructors.
    /// For example `if foo { 1 }` shouldn't have `foo { 1 }` as the condition, but `foo` instead.
    pub(crate) fn parse_expression_no_constructors_or_error(&mut self) -> Expression {
        self.parse_expression_or_error_impl(false) // allow constructors
    }

    pub(crate) fn parse_expression_or_error_impl(
        &mut self,
        allow_constructors: bool,
    ) -> Expression {
        if let Some(expr) = self.parse_expression_impl(allow_constructors) {
            expr
        } else {
            self.push_expected_expression_after_this_error();
            Expression {
                kind: ExpressionKind::Error,
                span: Span::from(self.previous_token_span.end()..self.previous_token_span.end()),
            }
        }
    }

    fn parse_expression_impl(&mut self, allow_constructors: bool) -> Option<Expression> {
        self.parse_equal_or_not_equal(allow_constructors)
    }

    pub(super) fn parse_term(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_span = self.current_token_span;
        if let Some(operator) = self.parse_unary_op() {
            let Some(rhs) = self.parse_term(allow_constructors) else {
                self.push_error(
                    ParserErrorReason::ExpectedExpressionAfterThis,
                    self.previous_token_span,
                );
                return None;
            };
            let kind = ExpressionKind::Prefix(Box::new(PrefixExpression { operator, rhs }));
            let span = self.span_since(start_span);
            return Some(Expression { kind, span });
        }

        self.parse_atom_or_unary_right(allow_constructors)
    }

    fn parse_unary_op(&mut self) -> Option<UnaryOp> {
        if self.token.token() == &Token::Ampersand
            && self.next_token.token() == &Token::Keyword(Keyword::Mut)
        {
            self.next_token();
            self.next_token();
            Some(UnaryOp::MutableReference)
        } else if self.eat(Token::Minus) {
            Some(UnaryOp::Minus)
        } else if self.eat(Token::Bang) {
            Some(UnaryOp::Not)
        } else if self.eat(Token::Star) {
            Some(UnaryOp::Dereference { implicitly_added: false })
        } else {
            None
        }
    }

    fn parse_atom_or_unary_right(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_span = self.current_token_span;
        let mut atom = self.parse_atom(allow_constructors)?;

        loop {
            let is_macro_call =
                self.token.token() == &Token::Bang && self.next_token.token() == &Token::LeftParen;
            if is_macro_call {
                // Next `self.parse_arguments` will return `Some(...)`
                self.next_token();
            }

            if let Some(arguments) = self.parse_arguments() {
                let kind = ExpressionKind::Call(Box::new(CallExpression {
                    func: Box::new(atom),
                    arguments,
                    is_macro_call,
                }));
                let span = self.span_since(start_span);
                atom = Expression { kind, span };
                continue;
            }

            if self.eat_dot() {
                let field_name = if let Some(ident) = self.eat_ident() {
                    ident
                } else if let Some(int) = self.eat_int() {
                    Ident::new(int.to_string(), self.previous_token_span)
                } else {
                    self.push_error(
                        ParserErrorReason::ExpectedIdentifierAfterDot,
                        self.current_token_span,
                    );
                    continue;
                };

                let generics = if self.eat_double_colon() {
                    let generics = self.parse_path_generics();
                    if generics.is_none() {
                        // TODO: error (found `::` but not `::<...>`)
                    }
                    generics
                } else {
                    None
                };

                let is_macro_call = self.token.token() == &Token::Bang
                    && self.next_token.token() == &Token::LeftParen;
                if is_macro_call {
                    // Next `self.parse_arguments` will return `Some(...)`
                    self.next_token();
                }

                if let Some(arguments) = self.parse_arguments() {
                    let kind = ExpressionKind::MethodCall(Box::new(MethodCallExpression {
                        object: atom,
                        method_name: field_name,
                        generics,
                        arguments,
                        is_macro_call,
                    }));
                    let span = self.span_since(start_span);
                    atom = Expression { kind, span };
                } else {
                    let kind = ExpressionKind::MemberAccess(Box::new(MemberAccessExpression {
                        lhs: atom,
                        rhs: field_name,
                    }));
                    let span = self.span_since(start_span);
                    atom = Expression { kind, span };
                }
                continue;
            }

            if self.eat_keyword(Keyword::As) {
                let typ = self.parse_type_or_error();
                let kind =
                    ExpressionKind::Cast(Box::new(CastExpression { lhs: atom, r#type: typ }));
                let span = self.span_since(start_span);
                atom = Expression { kind, span };
                continue;
            }

            if self.eat_left_bracket() {
                let index = self.parse_expression_or_error();
                if !self.eat_right_bracket() {
                    // TODO: error
                }
                let kind =
                    ExpressionKind::Index(Box::new(IndexExpression { collection: atom, index }));
                let span = self.span_since(start_span);
                atom = Expression { kind, span };
                continue;
            }

            break;
        }

        Some(atom)
    }

    fn parse_atom(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_span = self.current_token_span;
        let kind = self.parse_atom_kind(allow_constructors)?;
        Some(Expression { kind, span: self.span_since(start_span) })
    }

    fn parse_atom_kind(&mut self, allow_constructors: bool) -> Option<ExpressionKind> {
        if let Some(literal) = self.parse_literal() {
            return Some(literal);
        }

        if let Some(kind) = self.parse_parentheses_expression() {
            return Some(kind);
        }

        if let Some(kind) = self.parse_unsafe_expr() {
            return Some(kind);
        }

        if let Some(kind) = self.parse_path_expr(allow_constructors) {
            return Some(kind);
        }

        if let Some(kind) = self.parse_if_expr() {
            return Some(kind);
        }

        if let Some(kind) = self.parse_lambda() {
            return Some(kind);
        }

        // TODO: parse these too
        // comptime_expr(statement.clone()),
        // unquote(expr_parser.clone()),
        // as_trait_path(parse_type()).map(ExpressionKind::AsTraitPath),
        // type_path(parse_type()),

        if let Some(kind) = self.parse_resolved_expr() {
            return Some(kind);
        }

        if let Some(kind) = self.parse_interned_expr() {
            return Some(kind);
        }

        if let Some(kind) = self.parse_interned_statement_expr() {
            return Some(kind);
        }

        None
    }

    fn parse_resolved_expr(&mut self) -> Option<ExpressionKind> {
        if let Some(token) = self.eat_kind(TokenKind::UnquoteMarker) {
            match token.into_token() {
                Token::UnquoteMarker(expr_id) => return Some(ExpressionKind::Resolved(expr_id)),
                _ => unreachable!(""),
            }
        }

        None
    }

    fn parse_interned_expr(&mut self) -> Option<ExpressionKind> {
        if let Some(token) = self.eat_kind(TokenKind::InternedExpr) {
            match token.into_token() {
                Token::InternedExpr(id) => return Some(ExpressionKind::Interned(id)),
                _ => unreachable!(""),
            }
        }

        None
    }

    fn parse_interned_statement_expr(&mut self) -> Option<ExpressionKind> {
        if let Some(token) = self.eat_kind(TokenKind::InternedStatement) {
            match token.into_token() {
                Token::InternedStatement(id) => return Some(ExpressionKind::InternedStatement(id)),
                _ => unreachable!(""),
            }
        }

        None
    }

    fn parse_unsafe_expr(&mut self) -> Option<ExpressionKind> {
        if !self.eat_keyword(Keyword::Unsafe) {
            return None;
        }

        let start_span = self.span_since(self.previous_token_span);
        if let Some(block) = self.parse_block_expression() {
            Some(ExpressionKind::Unsafe(block, self.span_since(start_span)))
        } else {
            Some(ExpressionKind::Error)
        }
    }

    fn parse_path_expr(&mut self, allow_constructors: bool) -> Option<ExpressionKind> {
        let path = self.parse_path();
        if path.is_empty() {
            return None;
        }

        if allow_constructors && self.eat_left_brace() {
            return Some(self.parse_constructor(path));
        }

        Some(ExpressionKind::Variable(path))
    }

    fn parse_constructor(&mut self, path: Path) -> ExpressionKind {
        let mut fields = Vec::new();
        let mut trailing_comma = false;

        loop {
            let start_span = self.current_token_span;
            let Some(ident) = self.eat_ident() else {
                if !self.eat_right_brace() {
                    // TODO: error
                }
                break;
            };

            if !trailing_comma && !fields.is_empty() {
                self.push_error(
                    ParserErrorReason::MissingCommaSeparatingConstructorFields,
                    start_span,
                );
            }

            if self.eat_colon() {
                let expression = self.parse_expression_or_error();
                fields.push((ident, expression));
            } else {
                fields.push((ident.clone(), ident.into()));
            }

            trailing_comma = self.eat_commas();

            if self.eat_right_brace() {
                break;
            }
        }

        ExpressionKind::Constructor(Box::new(ConstructorExpression {
            typ: UnresolvedType::from_path(path),
            fields,
            struct_type: None,
        }))
    }

    fn parse_if_expr(&mut self) -> Option<ExpressionKind> {
        if !self.eat_keyword(Keyword::If) {
            return None;
        }

        let condition = self.parse_expression_no_constructors_or_error();

        let start_span = self.current_token_span;
        let Some(consequence) = self.parse_block_expression() else {
            self.push_error(
                ParserErrorReason::ExpectedLeftBraceAfterIfCondition,
                self.current_token_span,
            );
            let span = Span::from(self.previous_token_span.end()..self.previous_token_span.end());
            return Some(ExpressionKind::If(Box::new(IfExpression {
                condition,
                consequence: Expression { kind: ExpressionKind::Error, span },
                alternative: None,
            })));
        };
        let span = self.span_since(start_span);
        let consequence = Expression { kind: ExpressionKind::Block(consequence), span };

        let alternative = if self.eat_keyword(Keyword::Else) {
            let start_span = self.current_token_span;
            if let Some(alternative) = self.parse_block_expression() {
                let span = self.span_since(start_span);
                Some(Expression { kind: ExpressionKind::Block(alternative), span })
            } else if let Some(if_expr) = self.parse_if_expr() {
                Some(Expression { kind: if_expr, span: self.span_since(start_span) })
            } else {
                self.push_error(
                    ParserErrorReason::ExpectedLeftBraceOfIfAfterElse,
                    self.current_token_span,
                );
                None
            }
        } else {
            None
        };

        Some(ExpressionKind::If(Box::new(IfExpression { condition, consequence, alternative })))
    }

    fn parse_literal(&mut self) -> Option<ExpressionKind> {
        if let Some(bool) = self.eat_bool() {
            return Some(ExpressionKind::Literal(Literal::Bool(bool)));
        }

        if let Some(int) = self.eat_int() {
            return Some(ExpressionKind::integer(int));
        }

        if let Some(string) = self.eat_str() {
            return Some(ExpressionKind::Literal(Literal::Str(string)));
        }

        if let Some((string, n)) = self.eat_raw_str() {
            return Some(ExpressionKind::Literal(Literal::RawStr(string, n)));
        }

        if let Some(string) = self.eat_fmt_str() {
            return Some(ExpressionKind::Literal(Literal::FmtStr(string)));
        }

        if let Some(tokens) = self.eat_quote() {
            return Some(ExpressionKind::Quote(tokens));
        }

        if let Some(kind) = self.parse_array_expression() {
            return Some(kind);
        }

        // Check if it's `&[`
        if self.token.token() == &Token::Ampersand && self.next_token.token() == &Token::LeftBracket
        {
            self.next_token();

            return Some(ExpressionKind::Literal(Literal::Slice(
                self.parse_array_literal(true).unwrap(),
            )));
        }

        if let Some(kind) = self.parse_block_expression() {
            return Some(ExpressionKind::Block(kind));
        }

        None
    }

    fn parse_array_expression(&mut self) -> Option<ExpressionKind> {
        self.parse_array_literal(false)
            .map(|array_literal| ExpressionKind::Literal(Literal::Array(array_literal)))
    }

    fn parse_array_literal(&mut self, is_slice: bool) -> Option<ArrayLiteral> {
        if !self.eat_left_bracket() {
            return None;
        }

        if self.eat_right_bracket() {
            return Some(ArrayLiteral::Standard(Vec::new()));
        }

        let first_expr = self.parse_expression_or_error();
        if first_expr.kind == ExpressionKind::Error {
            return Some(ArrayLiteral::Standard(Vec::new()));
        }

        if self.eat_semicolon() {
            let length = self.parse_expression_or_error();
            if !self.eat_right_bracket() {
                if is_slice {
                    self.push_error(
                        ParserErrorReason::ExpectedBracketAfterSlice,
                        self.current_token_span,
                    );
                } else {
                    self.push_error(
                        ParserErrorReason::ExpectedBracketAfterArray,
                        self.current_token_span,
                    );
                }
            }
            return Some(ArrayLiteral::Repeated {
                repeated_element: Box::new(first_expr),
                length: Box::new(length),
            });
        }

        let mut exprs = vec![first_expr];
        let mut trailing_comma = self.eat_comma();
        loop {
            if self.eat_right_bracket() {
                break;
            }

            let start_span = self.current_token_span;
            let Some(expr) = self.parse_expression() else {
                self.eat_right_brace();
                break;
            };

            if !trailing_comma {
                self.push_error(ParserErrorReason::MissingCommaSeparatingExpressions, start_span);
            }

            exprs.push(expr);

            trailing_comma = self.eat_commas();
        }

        Some(ArrayLiteral::Standard(exprs))
    }

    fn parse_parentheses_expression(&mut self) -> Option<ExpressionKind> {
        if !self.eat_left_paren() {
            return None;
        }

        if self.eat_right_paren() {
            return Some(ExpressionKind::Literal(Literal::Unit));
        }

        let mut exprs = Vec::new();
        let mut trailing_comma = false;
        loop {
            let start_span = self.current_token_span;
            let Some(expr) = self.parse_expression() else {
                self.push_error(
                    ParserErrorReason::ExpectedExpressionAfterThis,
                    self.previous_token_span,
                );
                self.eat_right_paren();
                break;
            };
            if !trailing_comma && !exprs.is_empty() {
                self.push_error(ParserErrorReason::MissingCommaSeparatingExpressions, start_span);
            }

            exprs.push(expr);

            trailing_comma = self.eat_commas();

            if self.eat_right_paren() {
                break;
            }
        }

        Some(if exprs.len() == 1 && !trailing_comma {
            ExpressionKind::Parenthesized(Box::new(exprs.remove(0)))
        } else {
            ExpressionKind::Tuple(exprs)
        })
    }

    pub(super) fn parse_block_expression(&mut self) -> Option<BlockExpression> {
        if !self.eat_left_brace() {
            return None;
        }

        let mut statements = Vec::new();

        loop {
            if self.eat_right_brace() {
                break;
            }

            let Some(statement) = self.parse_statement() else {
                // TODO: error?
                self.eat_right_brace();
                break;
            };

            statements.push(statement);

            // TODO: error if missing semicolon and statement requires one and is not the last one in the block
            self.eat_semicolons();
        }

        Some(BlockExpression { statements })
    }

    pub(super) fn push_expected_expression_after_this_error(&mut self) {
        self.push_error(ParserErrorReason::ExpectedExpressionAfterThis, self.previous_token_span);
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::{
        ast::{
            ArrayLiteral, BinaryOpKind, ExpressionKind, Literal, StatementKind, UnaryOp,
            UnresolvedTypeData,
        },
        parser::{
            parser::tests::{get_single_error, get_source_with_error_span},
            Parser, ParserErrorReason,
        },
    };

    #[test]
    fn parses_bool_literals() {
        let src = "true";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        assert!(matches!(expr.kind, ExpressionKind::Literal(Literal::Bool(true))));

        let src = "false";
        let expr = Parser::for_str(src).parse_expression_or_error();
        assert!(matches!(expr.kind, ExpressionKind::Literal(Literal::Bool(false))));
    }

    #[test]
    fn parses_integer_literal() {
        let src = "42";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 42_u128.into());
        assert!(!negative);
    }

    #[test]
    fn parses_parenthesized_expression() {
        let src = "(42)";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Parenthesized(expr) = expr.kind else {
            panic!("Expected parenthesized expression");
        };
        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 42_u128.into());
        assert!(!negative);
    }

    #[test]
    fn parses_unit() {
        let src = "()";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        assert!(matches!(expr.kind, ExpressionKind::Literal(Literal::Unit)));
    }

    #[test]
    fn parses_str() {
        let src = "\"hello\"";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Literal(Literal::Str(string)) = expr.kind else {
            panic!("Expected string literal");
        };
        assert_eq!(string, "hello");
    }

    #[test]
    fn parses_raw_str() {
        let src = "r#\"hello\"#";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Literal(Literal::RawStr(string, n)) = expr.kind else {
            panic!("Expected raw string literal");
        };
        assert_eq!(string, "hello");
        assert_eq!(n, 1);
    }

    #[test]
    fn parses_fmt_str() {
        let src = "f\"hello\"";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Literal(Literal::FmtStr(string)) = expr.kind else {
            panic!("Expected format string literal");
        };
        assert_eq!(string, "hello");
    }

    #[test]
    fn parses_tuple_expression() {
        let src = "(1, 2)";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Tuple(mut exprs) = expr.kind else {
            panic!("Expected tuple expression");
        };
        assert_eq!(exprs.len(), 2);

        let expr = exprs.remove(0);
        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 1_u128.into());
        assert!(!negative);

        let expr = exprs.remove(0);
        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 2_u128.into());
        assert!(!negative);
    }

    #[test]
    fn parses_block_expression_with_a_single_expression() {
        let src = "{ 1 }";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Block(mut block) = expr.kind else {
            panic!("Expected block expression");
        };
        assert_eq!(block.statements.len(), 1);

        let statement = block.statements.remove(0);
        let StatementKind::Expression(expr) = statement.kind else {
            panic!("Expected expression statement");
        };

        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 1_u128.into());
        assert!(!negative);
    }

    #[test]
    fn parses_block_expression_with_multiple_statements() {
        let src = "
        {
            let x = 1;
            let y = 2;
            3
        }
        ";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Block(block) = expr.kind else {
            panic!("Expected block expression");
        };
        assert_eq!(block.statements.len(), 3);
        assert_eq!(block.statements[0].kind.to_string(), "let x = 1");
        assert_eq!(block.statements[1].kind.to_string(), "let y = 2");
        assert_eq!(block.statements[2].kind.to_string(), "3");
    }

    #[test]
    fn parses_unsafe_expression() {
        let src = "unsafe { 1 }";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Unsafe(block, _) = expr.kind else {
            panic!("Expected unsafe expression");
        };
        assert_eq!(block.statements.len(), 1);
    }

    #[test]
    fn parses_unclosed_parentheses() {
        let src = "
        (
        ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        parser.parse_expression();
        let reason = get_single_error(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::ExpectedExpressionAfterThis));
    }

    #[test]
    fn parses_missing_comma_in_tuple() {
        let src = "
        (1 2)
           ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        parser.parse_expression();
        let reason = get_single_error(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::MissingCommaSeparatingExpressions));
    }

    #[test]
    fn parses_empty_array_expression() {
        let src = "[]";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Literal(Literal::Array(ArrayLiteral::Standard(exprs))) = expr.kind
        else {
            panic!("Expected array literal");
        };
        assert!(exprs.is_empty());
    }

    #[test]
    fn parses_array_expression_with_one_element() {
        let src = "[1]";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Literal(Literal::Array(ArrayLiteral::Standard(exprs))) = expr.kind
        else {
            panic!("Expected array literal");
        };
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].to_string(), "1");
    }

    #[test]
    fn parses_array_expression_with_two_elements() {
        let src = "[1, 3]";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Literal(Literal::Array(ArrayLiteral::Standard(exprs))) = expr.kind
        else {
            panic!("Expected array literal");
        };
        assert_eq!(exprs.len(), 2);
        assert_eq!(exprs[0].to_string(), "1");
        assert_eq!(exprs[1].to_string(), "3");
    }

    #[test]
    fn parses_repeated_array_expression() {
        let src = "[1; 10]";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Literal(Literal::Array(ArrayLiteral::Repeated {
            repeated_element,
            length,
        })) = expr.kind
        else {
            panic!("Expected array literal");
        };
        assert_eq!(repeated_element.to_string(), "1");
        assert_eq!(length.to_string(), "10");
    }

    #[test]
    fn parses_empty_slice_expression() {
        let src = "&[]";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Literal(Literal::Slice(ArrayLiteral::Standard(exprs))) = expr.kind
        else {
            panic!("Expected slice literal");
        };
        assert!(exprs.is_empty());
    }

    #[test]
    fn parses_variable_ident() {
        let src = "foo";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Variable(path) = expr.kind else {
            panic!("Expected variable");
        };
        assert_eq!(path.to_string(), "foo");
    }

    #[test]
    fn parses_variable_path() {
        let src = "foo::bar";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Variable(path) = expr.kind else {
            panic!("Expected variable");
        };
        assert_eq!(path.to_string(), "foo::bar");
    }

    #[test]
    fn parses_mutable_ref() {
        let src = "&mut foo";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Prefix(prefix) = expr.kind else {
            panic!("Expected prefix expression");
        };
        assert!(matches!(prefix.operator, UnaryOp::MutableReference));

        let ExpressionKind::Variable(path) = prefix.rhs.kind else {
            panic!("Expected variable");
        };
        assert_eq!(path.to_string(), "foo");
    }

    #[test]
    fn parses_minus() {
        let src = "-foo";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Prefix(prefix) = expr.kind else {
            panic!("Expected prefix expression");
        };
        assert!(matches!(prefix.operator, UnaryOp::Minus));

        let ExpressionKind::Variable(path) = prefix.rhs.kind else {
            panic!("Expected variable");
        };
        assert_eq!(path.to_string(), "foo");
    }

    #[test]
    fn parses_not() {
        let src = "!foo";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Prefix(prefix) = expr.kind else {
            panic!("Expected prefix expression");
        };
        assert!(matches!(prefix.operator, UnaryOp::Not));

        let ExpressionKind::Variable(path) = prefix.rhs.kind else {
            panic!("Expected variable");
        };
        assert_eq!(path.to_string(), "foo");
    }

    #[test]
    fn parses_dereference() {
        let src = "*foo";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Prefix(prefix) = expr.kind else {
            panic!("Expected prefix expression");
        };
        assert!(matches!(prefix.operator, UnaryOp::Dereference { implicitly_added: false }));

        let ExpressionKind::Variable(path) = prefix.rhs.kind else {
            panic!("Expected variable");
        };
        assert_eq!(path.to_string(), "foo");
    }

    #[test]
    fn parses_quote() {
        let src = "quote { 1 }";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Quote(tokens) = expr.kind else {
            panic!("Expected quote expression");
        };
        assert_eq!(tokens.0.len(), 1);
    }

    #[test]
    fn parses_call() {
        let src = "foo(1, 2)";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Call(call) = expr.kind else {
            panic!("Expected call expression");
        };
        assert_eq!(call.func.to_string(), "foo");
        assert_eq!(call.arguments.len(), 2);
        assert!(!call.is_macro_call);
    }

    #[test]
    fn parses_call_with_turbofish() {
        let src = "foo::<T>(1, 2)";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Call(call) = expr.kind else {
            panic!("Expected call expression");
        };
        assert_eq!(call.func.to_string(), "foo::<T>");
        assert_eq!(call.arguments.len(), 2);
        assert!(!call.is_macro_call);
    }

    #[test]
    fn parses_macro_call() {
        let src = "foo!(1, 2)";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Call(call) = expr.kind else {
            panic!("Expected call expression");
        };
        assert_eq!(call.func.to_string(), "foo");
        assert_eq!(call.arguments.len(), 2);
        assert!(call.is_macro_call);
    }

    #[test]
    fn parses_member_access() {
        let src = "foo.bar";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::MemberAccess(member_access) = expr.kind else {
            panic!("Expected member access expression");
        };
        assert_eq!(member_access.lhs.to_string(), "foo");
        assert_eq!(member_access.rhs.to_string(), "bar");
    }

    #[test]
    fn parses_method_call() {
        let src = "foo.bar(1, 2)";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::MethodCall(method_call) = expr.kind else {
            panic!("Expected method call expression");
        };
        assert_eq!(method_call.object.to_string(), "foo");
        assert_eq!(method_call.method_name.to_string(), "bar");
        assert!(!method_call.is_macro_call);
        assert_eq!(method_call.arguments.len(), 2);
        assert!(method_call.generics.is_none());
    }

    #[test]
    fn parses_method_call_with_turbofish() {
        let src = "foo.bar::<T, U>(1, 2)";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::MethodCall(method_call) = expr.kind else {
            panic!("Expected method call expression");
        };
        assert_eq!(method_call.object.to_string(), "foo");
        assert_eq!(method_call.method_name.to_string(), "bar");
        assert!(!method_call.is_macro_call);
        assert_eq!(method_call.arguments.len(), 2);
        assert_eq!(method_call.generics.unwrap().len(), 2);
    }

    #[test]
    fn parses_method_macro_call() {
        let src = "foo.bar!(1, 2)";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::MethodCall(method_call) = expr.kind else {
            panic!("Expected method call expression");
        };
        assert_eq!(method_call.object.to_string(), "foo");
        assert_eq!(method_call.method_name.to_string(), "bar");
        assert!(method_call.is_macro_call);
        assert_eq!(method_call.arguments.len(), 2);
        assert!(method_call.generics.is_none());
    }

    #[test]
    fn parses_empty_constructor() {
        let src = "Foo {}";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Constructor(constructor) = expr.kind else {
            panic!("Expected constructor");
        };
        assert_eq!(constructor.typ.to_string(), "Foo");
        assert!(constructor.fields.is_empty());
    }

    #[test]
    fn parses_constructor_with_fields() {
        let src = "Foo { x: 1, y, z: 2 }";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert_eq!(expr.span.end() as usize, src.len());
        assert!(parser.errors.is_empty());
        let ExpressionKind::Constructor(mut constructor) = expr.kind else {
            panic!("Expected constructor");
        };
        assert_eq!(constructor.typ.to_string(), "Foo");
        assert_eq!(constructor.fields.len(), 3);

        let (name, expr) = constructor.fields.remove(0);
        assert_eq!(name.to_string(), "x");
        assert_eq!(expr.to_string(), "1");

        let (name, expr) = constructor.fields.remove(0);
        assert_eq!(name.to_string(), "y");
        assert_eq!(expr.to_string(), "y");

        let (name, expr) = constructor.fields.remove(0);
        assert_eq!(name.to_string(), "z");
        assert_eq!(expr.to_string(), "2");
    }

    #[test]
    fn parses_parses_if_true() {
        let src = "if true { 1 }";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::If(if_expr) = expr.kind else {
            panic!("Expected if");
        };
        assert_eq!(if_expr.condition.to_string(), "true");
        let ExpressionKind::Block(block_expr) = if_expr.consequence.kind else {
            panic!("Expected block");
        };
        assert_eq!(block_expr.statements.len(), 1);
        assert_eq!(block_expr.statements[0].kind.to_string(), "1");
        assert!(if_expr.alternative.is_none());
    }

    #[test]
    fn parses_parses_if_var() {
        let src = "if foo { 1 }";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::If(if_expr) = expr.kind else {
            panic!("Expected if");
        };
        assert_eq!(if_expr.condition.to_string(), "foo");
    }

    #[test]
    fn parses_parses_if_else() {
        let src = "if true { 1 } else { 2 }";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::If(if_expr) = expr.kind else {
            panic!("Expected if");
        };
        assert_eq!(if_expr.condition.to_string(), "true");
        assert!(if_expr.alternative.is_some());
    }

    #[test]
    fn parses_parses_if_else_if() {
        let src = "if true { 1 } else if false { 2 } else { 3 }";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::If(if_expr) = expr.kind else {
            panic!("Expected if");
        };
        assert_eq!(if_expr.condition.to_string(), "true");
        let ExpressionKind::If(..) = if_expr.alternative.unwrap().kind else {
            panic!("Expected if");
        };
    }

    #[test]
    fn parses_cast() {
        let src = "1 as u8";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Cast(cast_expr) = expr.kind else {
            panic!("Expected cast");
        };
        assert_eq!(cast_expr.lhs.to_string(), "1");
        assert_eq!(cast_expr.r#type.to_string(), "u8");
    }

    #[test]
    fn parses_cast_missing_type() {
        let src = "
        1 as
          ^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        parser.parse_expression();
        let reason = get_single_error(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::ExpectedTypeAfterThis));
    }

    #[test]
    fn parses_index() {
        let src = "1[2]";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Index(index_expr) = expr.kind else {
            panic!("Expected index");
        };
        assert_eq!(index_expr.collection.to_string(), "1");
        assert_eq!(index_expr.index.to_string(), "2");
    }

    #[test]
    fn parses_operators() {
        for operator in BinaryOpKind::iter() {
            let src = format!("1 {operator} 2");
            let mut parser = Parser::for_str(&src);
            let expr = parser.parse_expression_or_error();
            assert!(parser.errors.is_empty(), "Expected no errors for {operator}");
            let ExpressionKind::Infix(infix_expr) = expr.kind else {
                panic!("Expected infix for {operator}");
            };
            assert_eq!(infix_expr.lhs.to_string(), "1");
            assert_eq!(infix_expr.operator.contents, operator);
            assert_eq!(infix_expr.rhs.to_string(), "2");
        }
    }

    #[test]
    fn parses_operator_precedence() {
        let src = "1 + 2 * 3 + 4";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Infix(infix_expr) = expr.kind else {
            panic!("Expected infix");
        };
        assert_eq!(infix_expr.lhs.to_string(), "(1 + (2 * 3))");
        assert_eq!(infix_expr.operator.contents, BinaryOpKind::Add);
        assert_eq!(infix_expr.rhs.to_string(), "4");
    }

    #[test]
    fn parses_empty_lambda() {
        let src = "|| 1";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Lambda(lambda) = expr.kind else {
            panic!("Expected lambda");
        };
        assert!(lambda.parameters.is_empty());
        assert_eq!(lambda.body.to_string(), "1");
        assert!(matches!(lambda.return_type.typ, UnresolvedTypeData::Unspecified));
    }

    #[test]
    fn parses_lambda_with_arguments() {
        let src = "|x, y: Field| 1";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Lambda(mut lambda) = expr.kind else {
            panic!("Expected lambda");
        };
        assert_eq!(lambda.parameters.len(), 2);

        let (pattern, typ) = lambda.parameters.remove(0);
        assert_eq!(pattern.to_string(), "x");
        assert!(matches!(typ.typ, UnresolvedTypeData::Unspecified));

        let (pattern, typ) = lambda.parameters.remove(0);
        assert_eq!(pattern.to_string(), "y");
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_lambda_with_return_type() {
        let src = "|| -> Field 1";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Lambda(lambda) = expr.kind else {
            panic!("Expected lambda");
        };
        assert!(lambda.parameters.is_empty());
        assert_eq!(lambda.body.to_string(), "1");
        assert!(matches!(lambda.return_type.typ, UnresolvedTypeData::FieldElement));
    }
}
