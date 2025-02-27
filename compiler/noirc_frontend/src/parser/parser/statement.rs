use noirc_errors::{Located, Location};

use crate::{
    ast::{
        AssignStatement, BinaryOp, BinaryOpKind, Expression, ExpressionKind, ForBounds,
        ForLoopStatement, ForRange, Ident, InfixExpression, LValue, LetStatement, Statement,
        StatementKind, WhileStatement,
    },
    parser::{ParserErrorReason, labels::ParsingRuleLabel},
    token::{Attribute, Keyword, Token, TokenKind},
};

use super::Parser;

impl Parser<'_> {
    pub(crate) fn parse_statement_or_error(&mut self) -> Statement {
        if let Some((statement, (_token, _span))) = self.parse_statement() {
            statement
        } else {
            self.expected_label(ParsingRuleLabel::Statement);
            Statement {
                kind: StatementKind::Error,
                location: self.location_at_previous_token_end(),
            }
        }
    }

    /// Statement = Attributes StatementKind ';'?
    pub(crate) fn parse_statement(&mut self) -> Option<(Statement, (Option<Token>, Location))> {
        loop {
            // Like in Rust, we allow parsing doc comments on top of a statement but they always produce a warning.
            self.warn_on_outer_doc_comments();

            if !self.current_token_comments.is_empty() {
                self.statement_comments = Some(std::mem::take(&mut self.current_token_comments));
            } else {
                self.statement_comments = None;
            }

            let attributes = self.parse_attributes();
            let start_location = self.current_token_location;
            let kind = self.parse_statement_kind(attributes);
            self.statement_comments = None;

            let (semicolon_token, semicolon_location) = if self.at(Token::Semicolon) {
                let token = self.token.clone();
                self.bump();
                let location = token.location();

                (Some(token.into_token()), location)
            } else {
                (None, self.previous_token_location)
            };

            let location = self.location_since(start_location);

            if let Some(kind) = kind {
                let statement = Statement { kind, location };
                return Some((statement, (semicolon_token, semicolon_location)));
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
    ///     | ComptimeStatement
    ///     | ForStatement
    ///     | LoopStatement
    ///     | WhileStatement
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
        attributes: Vec<(Attribute, Location)>,
    ) -> Option<StatementKind> {
        let start_location = self.current_token_location;

        if let Some(token) = self.eat_kind(TokenKind::InternedStatement) {
            match token.into_token() {
                Token::InternedStatement(statement) => {
                    return Some(StatementKind::Interned(statement));
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
            self.push_error(ParserErrorReason::EarlyReturn, self.location_since(start_location));
            return Some(StatementKind::Error);
        }

        if self.at_keyword(Keyword::Let) {
            let let_statement = self.parse_let_statement(attributes)?;
            return Some(StatementKind::Let(let_statement));
        }

        if self.at_keyword(Keyword::Comptime) {
            return self.parse_comptime_statement(attributes);
        }

        if let Some(for_loop) = self.parse_for() {
            return Some(StatementKind::For(for_loop));
        }

        if let Some((block, span)) = self.parse_loop() {
            return Some(StatementKind::Loop(block, span));
        }

        if let Some(while_) = self.parse_while() {
            return Some(StatementKind::While(while_));
        }

        if let Some(kind) = self.parse_if_expr() {
            let location = self.location_since(start_location);
            return Some(StatementKind::Expression(Expression { kind, location }));
        }

        if let Some(kind) = self.parse_match_expr() {
            let location = self.location_since(start_location);
            return Some(StatementKind::Expression(Expression { kind, location }));
        }

        if let Some(block) = self.parse_block() {
            return Some(StatementKind::Expression(Expression {
                kind: ExpressionKind::Block(block),
                location: self.location_since(start_location),
            }));
        }

        if let Some(token) = self.eat_kind(TokenKind::InternedLValue) {
            match token.into_token() {
                Token::InternedLValue(lvalue) => {
                    let lvalue = LValue::Interned(lvalue, self.location_since(start_location));
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
                    expression.location,
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
                    self.location_since(start_location),
                );
                return Some(StatementKind::Assign(AssignStatement { lvalue, expression }));
            } else {
                self.push_error(
                    ParserErrorReason::InvalidLeftHandSideOfAssignment,
                    expression.location,
                );
            }
        }

        Some(StatementKind::Expression(expression))
    }

    fn next_is_op_assign(&mut self) -> Option<BinaryOp> {
        let start_location = self.current_token_location;
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
            Some(Located::from(self.location_since(start_location), operator))
        } else {
            None
        }
    }

    /// ForStatement = 'for' identifier 'in' ForRange Block
    fn parse_for(&mut self) -> Option<ForLoopStatement> {
        let start_location = self.current_token_location;

        if !self.eat_keyword(Keyword::For) {
            return None;
        }

        let Some(identifier) = self.eat_ident() else {
            self.expected_identifier();
            let identifier = Ident::default();
            return Some(self.empty_for_loop(identifier, start_location));
        };

        if !self.eat_keyword(Keyword::In) {
            self.expected_token(Token::Keyword(Keyword::In));
            return Some(self.empty_for_loop(identifier, start_location));
        }

        let range = self.parse_for_range();

        let block_start_location = self.current_token_location;
        let block = if let Some(block) = self.parse_block() {
            Expression {
                kind: ExpressionKind::Block(block),
                location: self.location_since(block_start_location),
            }
        } else {
            self.expected_token(Token::LeftBrace);
            Expression {
                kind: ExpressionKind::Error,
                location: self.location_since(block_start_location),
            }
        };

        Some(ForLoopStatement {
            identifier,
            range,
            block,
            location: self.location_since(start_location),
        })
    }

    /// LoopStatement = 'loop' Block
    fn parse_loop(&mut self) -> Option<(Expression, Location)> {
        let start_location = self.current_token_location;
        if !self.eat_keyword(Keyword::Loop) {
            return None;
        }

        let block_start_location = self.current_token_location;
        let block = if let Some(block) = self.parse_block() {
            Expression {
                kind: ExpressionKind::Block(block),
                location: self.location_since(block_start_location),
            }
        } else {
            self.expected_token(Token::LeftBrace);
            Expression {
                kind: ExpressionKind::Error,
                location: self.location_since(block_start_location),
            }
        };

        Some((block, start_location))
    }

    /// WhileStatement = 'while' ExpressionExceptConstructor Block
    fn parse_while(&mut self) -> Option<WhileStatement> {
        let start_location = self.current_token_location;
        if !self.eat_keyword(Keyword::While) {
            return None;
        }

        let condition = self.parse_expression_except_constructor_or_error();

        let block_start_location = self.current_token_location;
        let block = if let Some(block) = self.parse_block() {
            Expression {
                kind: ExpressionKind::Block(block),
                location: self.location_since(block_start_location),
            }
        } else {
            self.expected_token(Token::LeftBrace);
            Expression {
                kind: ExpressionKind::Error,
                location: self.location_since(block_start_location),
            }
        };

        Some(WhileStatement { condition, body: block, while_keyword_location: start_location })
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

    fn empty_for_loop(&mut self, identifier: Ident, start_location: Location) -> ForLoopStatement {
        ForLoopStatement {
            identifier,
            range: ForRange::Array(Expression {
                kind: ExpressionKind::Error,
                location: Location::dummy(),
            }),
            block: Expression { kind: ExpressionKind::Error, location: Location::dummy() },
            location: self.location_since(start_location),
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
        attributes: Vec<(Attribute, Location)>,
    ) -> Option<StatementKind> {
        let start_location = self.current_token_location;

        if !self.eat_keyword(Keyword::Comptime) {
            return None;
        }

        if let Some(kind) = self.parse_comptime_statement_kind(attributes) {
            return Some(StatementKind::Comptime(Box::new(Statement {
                kind,
                location: self.location_since(start_location),
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
        attributes: Vec<(Attribute, Location)>,
    ) -> Option<StatementKind> {
        let start_location = self.current_token_location;

        if let Some(block) = self.parse_block() {
            return Some(StatementKind::Expression(Expression {
                kind: ExpressionKind::Block(block),
                location: self.location_since(start_location),
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
    fn parse_let_statement(
        &mut self,
        attributes: Vec<(Attribute, Location)>,
    ) -> Option<LetStatement> {
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
            Expression { kind: ExpressionKind::Error, location: self.current_token_location }
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
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{ExpressionKind, ForRange, LValue, Statement, StatementKind, UnresolvedTypeData},
        parser::{
            Parser, ParserErrorReason,
            parser::tests::{
                expect_no_errors, get_single_error, get_single_error_reason,
                get_source_with_error_span,
            },
        },
    };

    fn parse_statement_no_errors(src: &str) -> Statement {
        let mut parser = Parser::for_str_with_dummy_file(src);
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
        let src = "// Safety: comment
        let x = unsafe { 1 };";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let statement = parser.parse_statement_or_error();
        assert!(parser.errors.is_empty());
        let StatementKind::Let(let_statement) = statement.kind else {
            panic!("Expected let statement");
        };
        assert_eq!(let_statement.pattern.to_string(), "x");
    }

    #[test]
    fn parses_let_statement_with_unsafe_doc_comment() {
        let src = "/// Safety: doc comment
        let x = unsafe { 1 };";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let (statement, _) = parser.parse_statement().unwrap();
        let StatementKind::Let(let_statement) = statement.kind else {
            panic!("Expected let statement");
        };
        assert_eq!(let_statement.pattern.to_string(), "x");
    }

    #[test]
    fn parses_let_statement_with_unsafe_after_some_other_comment() {
        let src = "// Top comment
        // Safety: comment
        let x = unsafe { 1 };";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let statement = parser.parse_statement_or_error();
        assert!(parser.errors.is_empty());
        let StatementKind::Let(let_statement) = statement.kind else {
            panic!("Expected let statement");
        };
        assert_eq!(let_statement.pattern.to_string(), "x");
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
        let src = "// Safety: test 
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
        let src = "// Safety: comment
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
        let mut parser = Parser::for_str_with_dummy_file(&src);
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
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let statement = parser.parse_statement_or_error();
        assert!(matches!(statement.kind, StatementKind::Let(..)));
        let error = get_single_error(&parser.errors, span);
        assert_eq!(error.to_string(), "Expected a statement but found ']'");
    }

    #[test]
    fn recovers_on_unknown_statement_followed_by_semicolon() {
        let src = " ] ;";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let statement = parser.parse_statement();
        assert!(statement.is_none());
        assert_eq!(parser.errors.len(), 2);
    }

    #[test]
    fn recovers_on_unknown_statement_followed_by_right_brace() {
        let src = " ] }";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let statement = parser.parse_statement();
        assert!(statement.is_none());
        assert_eq!(parser.errors.len(), 2);
    }

    #[test]
    fn parses_empty_loop() {
        let src = "loop { }";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let statement = parser.parse_statement_or_error();
        let StatementKind::Loop(block, location) = statement.kind else {
            panic!("Expected loop");
        };
        let ExpressionKind::Block(block) = block.kind else {
            panic!("Expected block");
        };
        assert!(block.statements.is_empty());
        assert_eq!(location.span.start(), 0);
        assert_eq!(location.span.end(), 4);
    }

    #[test]
    fn parses_loop_with_statements() {
        let src = "loop { 1; 2 }";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let statement = parser.parse_statement_or_error();
        let StatementKind::Loop(block, _) = statement.kind else {
            panic!("Expected loop");
        };
        let ExpressionKind::Block(block) = block.kind else {
            panic!("Expected block");
        };
        assert_eq!(block.statements.len(), 2);
    }

    #[test]
    fn parses_let_with_assert() {
        let src = "let _ = assert(true);";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let statement = parser.parse_statement_or_error();
        let StatementKind::Let(let_statement) = statement.kind else {
            panic!("Expected let");
        };
        assert!(matches!(let_statement.expression.kind, ExpressionKind::Constrain(..)));
    }

    #[test]
    fn parses_empty_while() {
        let src = "while true { }";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let statement = parser.parse_statement_or_error();
        let StatementKind::While(while_) = statement.kind else {
            panic!("Expected while");
        };
        let ExpressionKind::Block(block) = while_.body.kind else {
            panic!("Expected block");
        };
        assert!(block.statements.is_empty());
        assert_eq!(while_.while_keyword_location.span.start(), 0);
        assert_eq!(while_.while_keyword_location.span.end(), 5);

        assert_eq!(while_.condition.to_string(), "true");
    }

    #[test]
    fn parses_while_with_statements() {
        let src = "while true { 1; 2 }";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let statement = parser.parse_statement_or_error();
        let StatementKind::While(while_) = statement.kind else {
            panic!("Expected while");
        };
        let ExpressionKind::Block(block) = while_.body.kind else {
            panic!("Expected block");
        };
        assert_eq!(block.statements.len(), 2);
    }
}
