use noirc_errors::{Span, Spanned};

use crate::{
    ast::{
        AssignStatement, BinaryOp, BinaryOpKind, ConstrainKind, ConstrainStatement, Expression,
        ExpressionKind, ForBounds, ForLoopStatement, ForRange, Ident, InfixExpression, LValue,
        LetStatement, Statement, StatementKind,
    },
    parser::{labels::ParsingRuleLabel, ParserErrorReason},
    token::{Attribute, Keyword, Token, TokenKind},
};

use super::{Parser, StatementDocComments};

impl<'a> Parser<'a> {
    pub(crate) fn parse_statement_or_error(&mut self) -> Statement {
        if let Some((statement, (_token, _span))) = self.parse_statement() {
            statement
        } else {
            self.expected_label(ParsingRuleLabel::Statement);
            Statement { kind: StatementKind::Error, span: self.span_at_previous_token_end() }
        }
    }

    /// Statement = Attributes StatementKind ';'?
    pub(crate) fn parse_statement(&mut self) -> Option<(Statement, (Option<Token>, Span))> {
        loop {
            let span_before_doc_comments = self.current_token_span;
            let doc_comments = self.parse_outer_doc_comments();
            let span_after_doc_comments = self.current_token_span;
            if doc_comments.is_empty() {
                self.statement_doc_comments = None;
            } else {
                self.statement_doc_comments = Some(StatementDocComments {
                    doc_comments,
                    start_span: span_before_doc_comments,
                    end_span: span_after_doc_comments,
                    read: false,
                });
            }

            let attributes = self.parse_attributes();
            let start_span = self.current_token_span;
            let kind = self.parse_statement_kind(attributes);

            if let Some(statement_doc_comments) = &self.statement_doc_comments {
                if !statement_doc_comments.read {
                    self.push_error(
                        ParserErrorReason::DocCommentDoesNotDocumentAnything,
                        Span::from(
                            statement_doc_comments.start_span.start()
                                ..statement_doc_comments.end_span.start(),
                        ),
                    );
                }
            }

            self.statement_doc_comments = None;

            let (semicolon_token, semicolon_span) = if self.at(Token::Semicolon) {
                let token = self.token.clone();
                self.bump();
                let span = token.to_span();

                (Some(token.into_token()), span)
            } else {
                (None, self.previous_token_span)
            };

            let span = self.span_since(start_span);

            if let Some(kind) = kind {
                let statement = Statement { kind, span };
                return Some((statement, (semicolon_token, semicolon_span)));
            }

            self.expected_label(ParsingRuleLabel::Statement);

            if semicolon_token.is_some() || self.at(Token::RightBrace) || self.at_eof() {
                return None;
            } else {
                self.bump();
            }
        }
    }

    /// StatementKind
    ///     = BreakStatement
    ///     | ContinueStatement
    ///     | ReturnStatement
    ///     | LetStatement
    ///     | ConstrainStatement
    ///     | ComptimeStatement
    ///     | ForStatement
    ///     | IfStatement
    ///     | BlockStatement
    ///     | AssignStatement
    ///     | ExpressionStatement
    ///
    /// BreakStatement = 'break'
    ///
    /// ContinueStatement = 'continue'
    ///
    /// ReturnStatement = 'return' Expression?
    ///
    /// IfStatement = IfExpression
    ///
    /// BlockStatement = Block
    ///
    /// AssignStatement = Expression '=' Expression
    ///
    /// ExpressionStatement = Expression
    fn parse_statement_kind(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
    ) -> Option<StatementKind> {
        let start_span = self.current_token_span;

        if let Some(token) = self.eat_kind(TokenKind::InternedStatement) {
            match token.into_token() {
                Token::InternedStatement(statement) => {
                    return Some(StatementKind::Interned(statement))
                }
                _ => unreachable!(),
            }
        }

        if self.eat_keyword(Keyword::Break) {
            return Some(StatementKind::Break);
        }

        if self.eat_keyword(Keyword::Continue) {
            return Some(StatementKind::Continue);
        }

        if self.eat_keyword(Keyword::Return) {
            self.parse_expression();
            self.push_error(ParserErrorReason::EarlyReturn, self.span_since(start_span));
            return Some(StatementKind::Error);
        }

        if self.at_keyword(Keyword::Let) {
            let let_statement = self.parse_let_statement(attributes)?;
            return Some(StatementKind::Let(let_statement));
        }

        if let Some(constrain) = self.parse_constrain_statement() {
            return Some(StatementKind::Constrain(constrain));
        }

        if self.at_keyword(Keyword::Comptime) {
            return self.parse_comptime_statement(attributes);
        }

        if let Some(for_loop) = self.parse_for() {
            return Some(StatementKind::For(for_loop));
        }

        if let Some(kind) = self.parse_if_expr() {
            return Some(StatementKind::Expression(Expression {
                kind,
                span: self.span_since(start_span),
            }));
        }

        if let Some(block) = self.parse_block() {
            return Some(StatementKind::Expression(Expression {
                kind: ExpressionKind::Block(block),
                span: self.span_since(start_span),
            }));
        }

        if let Some(token) = self.eat_kind(TokenKind::InternedLValue) {
            match token.into_token() {
                Token::InternedLValue(lvalue) => {
                    let lvalue = LValue::Interned(lvalue, self.span_since(start_span));
                    self.eat_or_error(Token::Assign);
                    let expression = self.parse_expression_or_error();
                    return Some(StatementKind::Assign(AssignStatement { lvalue, expression }));
                }
                _ => unreachable!(),
            }
        }

        let expression = self.parse_expression()?;

        if self.eat_assign() {
            if let Some(lvalue) = LValue::from_expression(expression.clone()) {
                let expression = self.parse_expression_or_error();
                return Some(StatementKind::Assign(AssignStatement { lvalue, expression }));
            } else {
                self.push_error(
                    ParserErrorReason::InvalidLeftHandSideOfAssignment,
                    expression.span,
                );
            }
        }

        if let Some(operator) = self.next_is_op_assign() {
            if let Some(lvalue) = LValue::from_expression(expression.clone()) {
                // Desugar `a <op>= b` to `a = a <op> b`. This relies on the evaluation of `a` having no side effects,
                // which is currently enforced by the restricted syntax of LValues.
                let infix = InfixExpression {
                    lhs: expression,
                    operator,
                    rhs: self.parse_expression_or_error(),
                };
                let expression = Expression::new(
                    ExpressionKind::Infix(Box::new(infix)),
                    self.span_since(start_span),
                );
                return Some(StatementKind::Assign(AssignStatement { lvalue, expression }));
            } else {
                self.push_error(
                    ParserErrorReason::InvalidLeftHandSideOfAssignment,
                    expression.span,
                );
            }
        }

        Some(StatementKind::Expression(expression))
    }

    fn next_is_op_assign(&mut self) -> Option<BinaryOp> {
        let start_span = self.current_token_span;
        let operator = if self.next_is(Token::Assign) {
            match self.token.token() {
                Token::Plus => Some(BinaryOpKind::Add),
                Token::Minus => Some(BinaryOpKind::Subtract),
                Token::Star => Some(BinaryOpKind::Multiply),
                Token::Slash => Some(BinaryOpKind::Divide),
                Token::Percent => Some(BinaryOpKind::Modulo),
                Token::Ampersand => Some(BinaryOpKind::And),
                Token::Caret => Some(BinaryOpKind::Xor),
                Token::ShiftLeft => Some(BinaryOpKind::ShiftLeft),
                Token::Pipe => Some(BinaryOpKind::Or),
                _ => None,
            }
        } else if self.at(Token::Greater) && self.next_is(Token::GreaterEqual) {
            // >>=
            Some(BinaryOpKind::ShiftRight)
        } else {
            None
        };

        if let Some(operator) = operator {
            self.bump();
            self.bump();
            Some(Spanned::from(self.span_since(start_span), operator))
        } else {
            None
        }
    }

    /// ForStatement = 'for' identifier 'in' ForRange Block
    fn parse_for(&mut self) -> Option<ForLoopStatement> {
        let start_span = self.current_token_span;

        if !self.eat_keyword(Keyword::For) {
            return None;
        }

        let Some(identifier) = self.eat_ident() else {
            self.expected_identifier();
            let identifier = Ident::default();
            return Some(self.empty_for_loop(identifier, start_span));
        };

        if !self.eat_keyword(Keyword::In) {
            self.expected_token(Token::Keyword(Keyword::In));
            return Some(self.empty_for_loop(identifier, start_span));
        }

        let range = self.parse_for_range();

        let block_start_span = self.current_token_span;
        let block = if let Some(block) = self.parse_block() {
            Expression {
                kind: ExpressionKind::Block(block),
                span: self.span_since(block_start_span),
            }
        } else {
            self.expected_token(Token::LeftBrace);
            Expression { kind: ExpressionKind::Error, span: self.span_since(block_start_span) }
        };

        Some(ForLoopStatement { identifier, range, block, span: self.span_since(start_span) })
    }

    /// ForRange
    ///     = ExpressionExceptConstructor
    ///     | ExpressionExceptConstructor '..' ExpressionExceptConstructor
    fn parse_for_range(&mut self) -> ForRange {
        let expr = self.parse_expression_except_constructor_or_error();

        if self.eat(Token::DoubleDot) {
            let end = self.parse_expression_except_constructor_or_error();
            ForRange::Range(ForBounds { start: expr, end, inclusive: false })
        } else if self.eat(Token::DoubleDotEqual) {
            let end = self.parse_expression_except_constructor_or_error();
            ForRange::Range(ForBounds { start: expr, end, inclusive: true })
        } else {
            ForRange::Array(expr)
        }
    }

    fn empty_for_loop(&mut self, identifier: Ident, start_span: Span) -> ForLoopStatement {
        ForLoopStatement {
            identifier,
            range: ForRange::Array(Expression {
                kind: ExpressionKind::Error,
                span: Span::default(),
            }),
            block: Expression { kind: ExpressionKind::Error, span: Span::default() },
            span: self.span_since(start_span),
        }
    }

    /// ComptimeStatement
    ///     = ComptimeBlock
    ///     | ComptimeLet
    ///     | ComptimeFor
    ///
    /// ComptimeBlock = 'comptime' Block
    ///
    /// ComptimeLet = 'comptime' LetStatement
    ///
    /// ComptimeFor = 'comptime' ForStatement
    fn parse_comptime_statement(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
    ) -> Option<StatementKind> {
        let start_span = self.current_token_span;

        if !self.eat_keyword(Keyword::Comptime) {
            return None;
        }

        if let Some(kind) = self.parse_comptime_statement_kind(attributes) {
            return Some(StatementKind::Comptime(Box::new(Statement {
                kind,
                span: self.span_since(start_span),
            })));
        }

        self.expected_one_of_tokens(&[
            Token::LeftBrace,
            Token::Keyword(Keyword::Let),
            Token::Keyword(Keyword::For),
        ]);

        None
    }

    fn parse_comptime_statement_kind(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
    ) -> Option<StatementKind> {
        let start_span = self.current_token_span;

        if let Some(block) = self.parse_block() {
            return Some(StatementKind::Expression(Expression {
                kind: ExpressionKind::Block(block),
                span: self.span_since(start_span),
            }));
        }

        if let Some(let_statement) = self.parse_let_statement(attributes) {
            return Some(StatementKind::Let(let_statement));
        }

        if let Some(for_loop) = self.parse_for() {
            return Some(StatementKind::For(for_loop));
        }

        None
    }

    /// LetStatement = 'let' pattern OptionalTypeAnnotation '=' Expression
    fn parse_let_statement(&mut self, attributes: Vec<(Attribute, Span)>) -> Option<LetStatement> {
        if !self.eat_keyword(Keyword::Let) {
            return None;
        }

        let attributes = self.validate_secondary_attributes(attributes);
        let pattern = self.parse_pattern_or_error();
        let r#type = self.parse_optional_type_annotation();
        let expression = if self.eat_assign() {
            self.parse_expression_or_error()
        } else {
            self.expected_token(Token::Assign);
            Expression { kind: ExpressionKind::Error, span: self.current_token_span }
        };

        Some(LetStatement {
            pattern,
            r#type,
            expression,
            attributes,
            comptime: false,
            is_global_let: false,
        })
    }

    /// ConstrainStatement
    ///     = 'constrain' Expression
    ///     | 'assert' Arguments
    ///     | 'assert_eq' Arguments
    fn parse_constrain_statement(&mut self) -> Option<ConstrainStatement> {
        let start_span = self.current_token_span;
        let kind = self.parse_constrain_kind()?;

        Some(match kind {
            ConstrainKind::Assert | ConstrainKind::AssertEq => {
                let arguments = self.parse_arguments();
                if arguments.is_none() {
                    self.expected_token(Token::LeftParen);
                }
                let arguments = arguments.unwrap_or_default();

                ConstrainStatement { kind, arguments, span: self.span_since(start_span) }
            }
            ConstrainKind::Constrain => {
                self.push_error(ParserErrorReason::ConstrainDeprecated, self.previous_token_span);

                let expression = self.parse_expression_or_error();
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
        ast::{
            ConstrainKind, ExpressionKind, ForRange, LValue, Statement, StatementKind,
            UnresolvedTypeData,
        },
        parser::{
            parser::tests::{
                expect_no_errors, get_single_error, get_single_error_reason,
                get_source_with_error_span,
            },
            Parser, ParserErrorReason,
        },
    };

    fn parse_statement_no_errors(src: &str) -> Statement {
        let mut parser = Parser::for_str(src);
        let statement = parser.parse_statement_or_error();
        expect_no_errors(&parser.errors);
        statement
    }

    #[test]
    fn parses_break() {
        let src = "break";
        let statement = parse_statement_no_errors(src);
        assert!(matches!(statement.kind, StatementKind::Break));
    }

    #[test]
    fn parses_continue() {
        let src = "continue";
        let statement = parse_statement_no_errors(src);
        assert!(matches!(statement.kind, StatementKind::Continue));
    }

    #[test]
    fn parses_let_statement_no_type() {
        let src = "let x = 1;";
        let statement = parse_statement_no_errors(src);
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
        let statement = parse_statement_no_errors(src);
        let StatementKind::Let(let_statement) = statement.kind else {
            panic!("Expected let statement");
        };
        assert_eq!(let_statement.pattern.to_string(), "x");
        assert_eq!(let_statement.r#type.to_string(), "Field");
        assert_eq!(let_statement.expression.to_string(), "1");
        assert!(!let_statement.comptime);
    }

    #[test]
    fn parses_let_statement_with_unsafe() {
        let src = "/// Safety: doc comment
        let x = unsafe { 1 };";
        let statement = parse_statement_no_errors(src);
        let StatementKind::Let(let_statement) = statement.kind else {
            panic!("Expected let statement");
        };
        assert_eq!(let_statement.pattern.to_string(), "x");
    }

    #[test]
    fn parses_assert() {
        let src = "assert(true, \"good\")";
        let statement = parse_statement_no_errors(src);
        let StatementKind::Constrain(constrain) = statement.kind else {
            panic!("Expected constrain statement");
        };
        assert_eq!(constrain.kind, ConstrainKind::Assert);
        assert_eq!(constrain.arguments.len(), 2);
    }

    #[test]
    fn parses_assert_eq() {
        let src = "assert_eq(1, 2, \"bad\")";
        let statement = parse_statement_no_errors(src);
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
        let statement = parser.parse_statement_or_error();
        let StatementKind::Constrain(constrain) = statement.kind else {
            panic!("Expected constrain statement");
        };
        assert_eq!(constrain.kind, ConstrainKind::Constrain);
        assert_eq!(constrain.arguments.len(), 1);

        let reason = get_single_error_reason(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::ConstrainDeprecated));
    }

    #[test]
    fn parses_comptime_block() {
        let src = "comptime { 1 }";
        let statement = parse_statement_no_errors(src);
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
        let statement = parse_statement_no_errors(src);
        let StatementKind::Comptime(statement) = statement.kind else {
            panic!("Expected comptime statement");
        };
        let StatementKind::Let(..) = statement.kind else {
            panic!("Expected let statement");
        };
    }

    #[test]
    fn parses_for_array() {
        let src = "for i in x { }";
        let statement = parse_statement_no_errors(src);
        let StatementKind::For(for_loop) = statement.kind else {
            panic!("Expected for loop");
        };
        assert_eq!(for_loop.identifier.to_string(), "i");
        let ForRange::Array(expr) = for_loop.range else {
            panic!("Expected array");
        };
        assert_eq!(expr.to_string(), "x");
    }

    #[test]
    fn parses_for_range() {
        let src = "for i in 0..10 { }";
        let statement = parse_statement_no_errors(src);
        let StatementKind::For(for_loop) = statement.kind else {
            panic!("Expected for loop");
        };
        assert_eq!(for_loop.identifier.to_string(), "i");
        let ForRange::Range(bounds) = for_loop.range else {
            panic!("Expected range");
        };
        assert_eq!(bounds.start.to_string(), "0");
        assert_eq!(bounds.end.to_string(), "10");
        assert!(!bounds.inclusive);
    }

    #[test]
    fn parses_for_range_inclusive() {
        let src = "for i in 0..=10 { }";
        let statement = parse_statement_no_errors(src);
        let StatementKind::For(for_loop) = statement.kind else {
            panic!("Expected for loop");
        };
        assert_eq!(for_loop.identifier.to_string(), "i");
        let ForRange::Range(bounds) = for_loop.range else {
            panic!("Expected range");
        };
        assert_eq!(bounds.start.to_string(), "0");
        assert_eq!(bounds.end.to_string(), "10");
        assert!(bounds.inclusive);
    }

    #[test]
    fn parses_comptime_for() {
        let src = "comptime for i in x { }";
        let statement = parse_statement_no_errors(src);
        let StatementKind::Comptime(statement) = statement.kind else {
            panic!("Expected comptime");
        };
        let StatementKind::For(for_loop) = statement.kind else {
            panic!("Expected for loop");
        };
        assert_eq!(for_loop.identifier.to_string(), "i");
        assert!(matches!(for_loop.range, ForRange::Array(..)));
    }

    #[test]
    fn parses_assignment() {
        let src = "x = 1";
        let statement = parse_statement_no_errors(src);
        let StatementKind::Assign(assign) = statement.kind else {
            panic!("Expected assign");
        };
        let LValue::Ident(ident) = assign.lvalue else {
            panic!("Expected ident");
        };
        assert_eq!(ident.to_string(), "x");
        assert_eq!(assign.expression.to_string(), "1");
    }

    #[test]
    fn parses_assignment_with_parentheses() {
        let src = "(x)[0] = 1";
        let statement = parse_statement_no_errors(src);
        let StatementKind::Assign(..) = statement.kind else {
            panic!("Expected assign");
        };
    }

    #[test]
    fn parses_assignment_with_unsafe() {
        let src = "/// Safety: test 
        x = unsafe { 1 }";
        let statement = parse_statement_no_errors(src);
        let StatementKind::Assign(assign) = statement.kind else {
            panic!("Expected assign");
        };
        let LValue::Ident(ident) = assign.lvalue else {
            panic!("Expected ident");
        };
        assert_eq!(ident.to_string(), "x");
    }

    #[test]
    fn parses_op_assignment() {
        let src = "x += 1";
        let statement = parse_statement_no_errors(src);
        let StatementKind::Assign(assign) = statement.kind else {
            panic!("Expected assign");
        };
        assert_eq!(assign.to_string(), "x = (x + 1)");
    }

    #[test]
    fn parses_op_assignment_with_shift_right() {
        let src = "x >>= 1";
        let statement = parse_statement_no_errors(src);
        let StatementKind::Assign(assign) = statement.kind else {
            panic!("Expected assign");
        };
        assert_eq!(assign.to_string(), "x = (x >> 1)");
    }

    #[test]
    fn parses_op_assignment_with_unsafe() {
        let src = "/// Safety: comment
        x += unsafe { 1 }";
        let statement = parse_statement_no_errors(src);
        let StatementKind::Assign(_) = statement.kind else {
            panic!("Expected assign");
        };
    }

    #[test]
    fn parses_if_statement_followed_by_tuple() {
        // This shouldn't be parsed as a call
        let src = "{ if 1 { 2 } (3, 4) }";
        let statement = parse_statement_no_errors(src);
        let StatementKind::Expression(expr) = statement.kind else {
            panic!("Expected expr");
        };
        let ExpressionKind::Block(block) = expr.kind else {
            panic!("Expected block");
        };
        assert_eq!(block.statements.len(), 2);
    }

    #[test]
    fn parses_block_followed_by_tuple() {
        // This shouldn't be parsed as a call
        let src = "{ { 2 } (3, 4) }";
        let statement = parse_statement_no_errors(src);
        let StatementKind::Expression(expr) = statement.kind else {
            panic!("Expected expr");
        };
        let ExpressionKind::Block(block) = expr.kind else {
            panic!("Expected block");
        };
        assert_eq!(block.statements.len(), 2);
    }

    #[test]
    fn errors_on_return_statement() {
        // This shouldn't be parsed as a call
        let src = "
        return 1
        ^^^^^^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        let statement = parser.parse_statement_or_error();
        assert!(matches!(statement.kind, StatementKind::Error));
        let reason = get_single_error_reason(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::EarlyReturn));
    }

    #[test]
    fn recovers_on_unknown_statement_followed_by_actual_statement() {
        let src = "
        ] let x = 1;
        ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        let statement = parser.parse_statement_or_error();
        assert!(matches!(statement.kind, StatementKind::Let(..)));
        let error = get_single_error(&parser.errors, span);
        assert_eq!(error.to_string(), "Expected a statement but found ']'");
    }

    #[test]
    fn recovers_on_unknown_statement_followed_by_semicolon() {
        let src = " ] ;";
        let mut parser = Parser::for_str(src);
        let statement = parser.parse_statement();
        assert!(statement.is_none());
        assert_eq!(parser.errors.len(), 2);
    }

    #[test]
    fn recovers_on_unknown_statement_followed_by_right_brace() {
        let src = " ] }";
        let mut parser = Parser::for_str(src);
        let statement = parser.parse_statement();
        assert!(statement.is_none());
        assert_eq!(parser.errors.len(), 2);
    }
}
