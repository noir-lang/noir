use noirc_errors::Span;

use crate::{
    ast::{
        ConstrainKind, ConstrainStatement, Expression, ExpressionKind, LetStatement, Statement,
        StatementKind,
    },
    parser::ParserErrorReason,
    token::{Attribute, Keyword, Token, TokenKind},
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_statement(&mut self) -> Statement {
        let attributes = self.parse_attributes();

        let start_span = self.current_token_span;
        let kind = self.parse_statement_kind(attributes);
        let span = self.span_since(start_span);
        Statement { kind, span }
    }

    fn parse_statement_kind(&mut self, attributes: Vec<(Attribute, Span)>) -> StatementKind {
        if let Some(token) = self.eat_kind(TokenKind::InternedStatement) {
            match token.into_token() {
                Token::InternedStatement(statement) => return StatementKind::Interned(statement),
                _ => unreachable!(),
            }
        }

        if self.eat_keyword(Keyword::Break) {
            return StatementKind::Break;
        }

        if self.eat_keyword(Keyword::Continue) {
            return StatementKind::Continue;
        }

        if self.token.token() == &Token::Keyword(Keyword::Let) {
            let let_statement = self.parse_let_statement(attributes).unwrap();
            return StatementKind::Let(let_statement);
        }

        if let Some(constrain) = self.parse_constrain_statement() {
            return StatementKind::Constrain(constrain);
        }

        if self.token.token() == &Token::Keyword(Keyword::Comptime) {
            return self.parse_comptime_statement(attributes).unwrap();
        }

        StatementKind::Expression(self.parse_expression())
    }

    fn parse_comptime_statement(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
    ) -> Option<StatementKind> {
        if !self.eat_keyword(Keyword::Comptime) {
            return None;
        }

        let start_span = self.current_token_span;

        if let Some(block) = self.parse_block_expression() {
            let span = self.span_since(start_span);
            return Some(StatementKind::Comptime(Box::new(Statement {
                kind: StatementKind::Expression(Expression::new(
                    ExpressionKind::Block(block),
                    span,
                )),
                span,
            })));
        }

        if let Some(let_statement) = self.parse_let_statement(attributes) {
            return Some(StatementKind::Comptime(Box::new(Statement {
                kind: StatementKind::Let(let_statement),
                span: self.span_since(start_span),
            })));
        }

        None
    }

    fn parse_let_statement(&mut self, attributes: Vec<(Attribute, Span)>) -> Option<LetStatement> {
        if !self.eat_keyword(Keyword::Let) {
            return None;
        }

        let attributes = self.validate_secondary_attributes(attributes);
        let pattern = self.parse_pattern();
        let r#type = self.parse_optional_type_annotation();
        let expression = if self.eat_assign() {
            self.parse_expression()
        } else {
            // TODO: error
            Expression { kind: ExpressionKind::Error, span: self.current_token_span }
        };

        Some(LetStatement { pattern, r#type, expression, attributes, comptime: false })
    }

    fn parse_constrain_statement(&mut self) -> Option<ConstrainStatement> {
        let start_span = self.current_token_span;
        let Some(kind) = self.parse_constrain_kind() else {
            return None;
        };

        Some(match kind {
            ConstrainKind::Assert | ConstrainKind::AssertEq => {
                let arguments = self.parse_arguments();
                if arguments.is_none() {
                    // TODO: error (expected arguments to assert/assert_eq)
                }
                let arguments = arguments.unwrap_or_default();

                ConstrainStatement { kind, arguments, span: self.span_since(start_span) }
            }
            ConstrainKind::Constrain => {
                self.push_error(ParserErrorReason::ConstrainDeprecated, self.previous_token_span);

                let expression = self.parse_expression();
                ConstrainStatement {
                    kind,
                    arguments: vec![expression],
                    span: self.span_since(start_span),
                }
            }
        })
    }

    fn parse_constrain_kind(&mut self) -> Option<ConstrainKind> {
        if self.eat_keyword(Keyword::Assert) {
            Some(ConstrainKind::Assert)
        } else if self.eat_keyword(Keyword::AssertEq) {
            Some(ConstrainKind::AssertEq)
        } else if self.eat_keyword(Keyword::Constrain) {
            Some(ConstrainKind::Constrain)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{ConstrainKind, ExpressionKind, StatementKind, UnresolvedTypeData},
        parser::{
            parser::tests::{get_single_error, get_source_with_error_span},
            Parser, ParserErrorReason,
        },
    };

    #[test]
    fn parses_break() {
        let src = "break";
        let mut parser = Parser::for_str(src);
        let statement = parser.parse_statement();
        assert!(parser.errors.is_empty());
        assert!(matches!(statement.kind, StatementKind::Break));
    }

    #[test]
    fn parses_continue() {
        let src = "continue";
        let mut parser = Parser::for_str(src);
        let statement = parser.parse_statement();
        assert!(parser.errors.is_empty());
        assert!(matches!(statement.kind, StatementKind::Continue));
    }

    #[test]
    fn parses_let_statement_no_type() {
        let src = "let x = 1;";
        let mut parser = Parser::for_str(src);
        let statement = parser.parse_statement();
        assert!(parser.errors.is_empty());
        let StatementKind::Let(let_statement) = statement.kind else {
            panic!("Expected let statement");
        };
        assert_eq!(let_statement.pattern.to_string(), "x");
        assert!(matches!(let_statement.r#type.typ, UnresolvedTypeData::Unspecified));
        assert_eq!(let_statement.expression.to_string(), "1");
        assert!(!let_statement.comptime);
    }

    #[test]
    fn parses_let_statement_with_type() {
        let src = "let x: Field = 1;";
        let mut parser = Parser::for_str(src);
        let statement = parser.parse_statement();
        assert!(parser.errors.is_empty());
        let StatementKind::Let(let_statement) = statement.kind else {
            panic!("Expected let statement");
        };
        assert_eq!(let_statement.pattern.to_string(), "x");
        assert_eq!(let_statement.r#type.to_string(), "Field");
        assert_eq!(let_statement.expression.to_string(), "1");
        assert!(!let_statement.comptime);
    }

    #[test]
    fn parses_assert() {
        let src = "assert(true, \"good\")";
        let mut parser = Parser::for_str(src);
        let statement = parser.parse_statement();
        assert!(parser.errors.is_empty());
        let StatementKind::Constrain(constrain) = statement.kind else {
            panic!("Expected constrain statement");
        };
        assert_eq!(constrain.kind, ConstrainKind::Assert);
        assert_eq!(constrain.arguments.len(), 2);
    }

    #[test]
    fn parses_assert_eq() {
        let src = "assert_eq(1, 2, \"bad\")";
        let mut parser = Parser::for_str(src);
        let statement = parser.parse_statement();
        assert!(parser.errors.is_empty());
        let StatementKind::Constrain(constrain) = statement.kind else {
            panic!("Expected constrain statement");
        };
        assert_eq!(constrain.kind, ConstrainKind::AssertEq);
        assert_eq!(constrain.arguments.len(), 3);
    }

    #[test]
    fn parses_constrain() {
        let src = "
        constrain 1
        ^^^^^^^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        let statement = parser.parse_statement();
        let StatementKind::Constrain(constrain) = statement.kind else {
            panic!("Expected constrain statement");
        };
        assert_eq!(constrain.kind, ConstrainKind::Constrain);
        assert_eq!(constrain.arguments.len(), 1);

        let reason = get_single_error(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::ConstrainDeprecated));
    }

    #[test]
    fn parses_comptime_block() {
        let src = "comptime { 1 }";
        let mut parser = Parser::for_str(&src);
        let statement = parser.parse_statement();
        assert!(parser.errors.is_empty());
        let StatementKind::Comptime(statement) = statement.kind else {
            panic!("Expected comptime statement");
        };
        let StatementKind::Expression(expr) = statement.kind else {
            panic!("Expected expression statement");
        };
        let ExpressionKind::Block(block) = expr.kind else {
            panic!("Expected block expression");
        };
        assert_eq!(block.statements.len(), 1);
    }

    #[test]
    fn parses_comptime_let() {
        let src = "comptime let x = 1;";
        let mut parser = Parser::for_str(&src);
        let statement = parser.parse_statement();
        assert!(parser.errors.is_empty());
        let StatementKind::Comptime(statement) = statement.kind else {
            panic!("Expected comptime statement");
        };
        let StatementKind::Let(..) = statement.kind else {
            panic!("Expected let statement");
        };
    }
}
