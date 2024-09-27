use crate::{
    ast::{ArrayLiteral, BlockExpression, Expression, ExpressionKind, Literal},
    parser::ParserErrorReason,
    token::Token,
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_expression(&mut self) -> Expression {
        let start_span = self.current_token_span;
        let kind = self.parse_expression_kind();
        let span = self.span_since(start_span);

        Expression { kind, span }
    }

    fn parse_expression_kind(&mut self) -> ExpressionKind {
        if let Some(bool) = self.eat_bool() {
            return ExpressionKind::Literal(Literal::Bool(bool));
        }

        if let Some(int) = self.eat_int() {
            return ExpressionKind::integer(int);
        }

        if let Some(string) = self.eat_str() {
            return ExpressionKind::Literal(Literal::Str(string));
        }

        if let Some((string, n)) = self.eat_raw_str() {
            return ExpressionKind::Literal(Literal::RawStr(string, n));
        }

        if let Some(string) = self.eat_fmt_str() {
            return ExpressionKind::Literal(Literal::FmtStr(string));
        }

        if let Some(kind) = self.parse_array_expression() {
            return kind;
        }

        if self.eat(Token::Ampersand) {
            if let Some(array_literal) = self.parse_array_literal(true) {
                return ExpressionKind::Literal(Literal::Slice(array_literal));
            }

            // TODO: parse this
            return ExpressionKind::Error;
        }

        if let Some(kind) = self.parse_parentheses_expression() {
            return kind;
        }

        if let Some(kind) = self.parse_block_expression() {
            return ExpressionKind::Block(kind);
        }

        let (path, trailing_double_colon) = self.parse_path_impl(false); // do not allow turbofish
        if !path.is_empty() {
            return ExpressionKind::Variable(path);
        }

        ExpressionKind::Error
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

        let start_span = self.current_token_span;
        let first_expr = self.parse_expression();
        if self.current_token_span == start_span {
            return Some(ArrayLiteral::Standard(Vec::new()));
        }

        if self.eat_semicolon() {
            let length = self.parse_expression();
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
            let expr = self.parse_expression();
            if self.current_token_span == start_span {
                self.eat_right_brace();
                break;
            }

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
            let expr = self.parse_expression();
            if let ExpressionKind::Error = expr.kind {
                self.push_error(ParserErrorReason::ExpectedExpression, start_span);
                self.eat_right_paren();
                break;
            }
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

            let start_span = self.current_token_span;
            let statement = self.parse_statement();
            if self.current_token_span == start_span {
                // TODO: error?
                self.eat_right_brace();
                break;
            }

            statements.push(statement);

            // TODO: error if missing semicolon and statement requires one and is not the last one in the block
            self.eat_semicolons();
        }

        Some(BlockExpression { statements })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{ArrayLiteral, ExpressionKind, Literal, StatementKind},
        parser::{
            parser::tests::{get_single_error, get_source_with_error_span},
            Parser, ParserErrorReason,
        },
    };

    #[test]
    fn parses_bool_literals() {
        let src = "true";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression();
        assert!(parser.errors.is_empty());
        assert!(matches!(expr.kind, ExpressionKind::Literal(Literal::Bool(true))));

        let src = "false";
        let expr = Parser::for_str(src).parse_expression();
        assert!(matches!(expr.kind, ExpressionKind::Literal(Literal::Bool(false))));
    }

    #[test]
    fn parses_integer_literal() {
        let src = "42";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression();
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
        let expr = parser.parse_expression();
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
        let expr = parser.parse_expression();
        assert!(parser.errors.is_empty());
        assert!(matches!(expr.kind, ExpressionKind::Literal(Literal::Unit)));
    }

    #[test]
    fn parses_str() {
        let src = "\"hello\"";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression();
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
        let expr = parser.parse_expression();
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
        let expr = parser.parse_expression();
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
        let expr = parser.parse_expression();
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
        let expr = parser.parse_expression();
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
        let expr = parser.parse_expression();
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
    fn parses_unclosed_parentheses() {
        let src = "
        (
        ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        parser.parse_expression();
        let reason = get_single_error(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::ExpectedExpression));
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
        let expr = parser.parse_expression();
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
        let expr = parser.parse_expression();
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
        let expr = parser.parse_expression();
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
        let expr = parser.parse_expression();
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
        let expr = parser.parse_expression();
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
        let expr = parser.parse_expression();
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
        let expr = parser.parse_expression();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Variable(path) = expr.kind else {
            panic!("Expected variable");
        };
        assert_eq!(path.to_string(), "foo::bar");
    }
}
