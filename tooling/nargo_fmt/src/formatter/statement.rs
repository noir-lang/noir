use noirc_frontend::{
    ast::{
        AssignStatement, ConstrainKind, ConstrainStatement, Expression, ExpressionKind,
        ForLoopStatement, ForRange, LetStatement, Statement, StatementKind, UnresolvedTypeData,
    },
    token::{Keyword, Token},
};

use super::{chunks::Chunks, Formatter};

impl<'a> Formatter<'a> {
    pub(super) fn format_statement(&mut self, statement: Statement, mut chunks: &mut Chunks) {
        chunks.leading_comment(self.skip_comments_and_whitespace_chunk());

        match statement.kind {
            StatementKind::Let(let_statement) => {
                chunks.group(self.format_let_statement(let_statement));
            }
            StatementKind::Constrain(constrain_statement) => {
                chunks.group(self.format_constrain_statement(constrain_statement))
            }
            StatementKind::Expression(expression) => match expression.kind {
                ExpressionKind::Block(block) => chunks.group(self.format_block_expression(
                    block, true, // force multiple lines
                )),
                ExpressionKind::Unsafe(block, _) => {
                    chunks.group(self.format_unsafe_expression(
                        block, true, // force multiple lines
                    ));
                }
                _ => self.format_expression(expression, &mut chunks),
            },
            StatementKind::Assign(assign_statement) => {
                chunks.group(self.format_assign(assign_statement));
            }
            StatementKind::For(for_loop_statement) => {
                chunks.group(self.format_for_loop(for_loop_statement));
            }
            StatementKind::Break => {
                chunks.text(self.chunk(|formatter| {
                    formatter.write_keyword(Keyword::Break);
                    formatter.write_semicolon();
                }));
            }
            StatementKind::Continue => {
                chunks.text(self.chunk(|formatter| {
                    formatter.write_keyword(Keyword::Continue);
                    formatter.write_semicolon();
                }));
            }
            StatementKind::Comptime(statement) => {
                chunks.group(self.format_comptime_statement(*statement));
            }
            StatementKind::Semi(expression) => {
                chunks.group(self.format_semi_statement(expression));
            }
            StatementKind::Interned(..) | StatementKind::Error => {
                unreachable!("Should not be present in the AST")
            }
        }
    }

    fn format_let_statement(&mut self, let_statement: LetStatement) -> Chunks {
        let mut chunks = Chunks::new();

        chunks.text(self.chunk(|formatter| {
            formatter.write_keyword(Keyword::Let);
            formatter.write_space();
            formatter.format_pattern(let_statement.pattern);
            if let_statement.r#type.typ != UnresolvedTypeData::Unspecified {
                formatter.write_token(Token::Colon);
                formatter.write_space();
                formatter.format_type(let_statement.r#type);
            }
            formatter.write_space();
            formatter.write_token(Token::Assign);
        }));
        chunks.increase_indentation();
        chunks.space_or_line();
        self.format_expression(let_statement.expression, &mut chunks);
        chunks.text(self.chunk(|formatter| {
            formatter.write_semicolon();
        }));
        chunks.decrease_indentation();

        chunks
    }

    fn format_constrain_statement(&mut self, constrain_statement: ConstrainStatement) -> Chunks {
        let mut chunks = Chunks::new();

        let keyword = match constrain_statement.kind {
            ConstrainKind::Assert => Keyword::Assert,
            ConstrainKind::AssertEq => Keyword::AssertEq,
            ConstrainKind::Constrain => {
                unreachable!("constrain always produces an error, and the formatter doesn't run when there are errors")
            }
        };

        chunks.text(self.chunk(|formatter| {
            formatter.write_keyword(keyword);
            formatter.write_left_paren();
        }));

        self.format_expressions_separated_by_comma(
            constrain_statement.arguments,
            false, // force trailing comma
            &mut chunks,
        );

        chunks.text(self.chunk(|formatter| {
            formatter.write_right_paren();
            formatter.write_semicolon();
        }));

        chunks
    }

    fn format_assign(&mut self, assign_statement: AssignStatement) -> Chunks {
        let mut chunks = Chunks::new();
        let mut is_op_assign = false;

        chunks.text(self.chunk(|formatter| {
            formatter.format_lvalue(assign_statement.lvalue);
            formatter.write_space();
            if formatter.token == Token::Assign {
                formatter.write_token(Token::Assign);
            } else {
                while formatter.token != Token::Assign {
                    formatter.write_current_token();
                    formatter.bump();
                    formatter.skip_comments_and_whitespace();
                }
                formatter.write_token(Token::Assign);
                is_op_assign = true;
            }
        }));
        chunks.increase_indentation();
        chunks.space_or_line();

        if is_op_assign {
            let ExpressionKind::Infix(infix) = assign_statement.expression.kind else {
                panic!("Expected an infix expression for op assign");
            };
            self.format_expression(infix.rhs, &mut chunks);
        } else {
            self.format_expression(assign_statement.expression, &mut chunks);
        }
        chunks.text(self.chunk(|formatter| {
            formatter.write_semicolon();
        }));
        chunks.decrease_indentation();
        chunks
    }

    fn format_for_loop(&mut self, for_loop: ForLoopStatement) -> Chunks {
        let mut chunks = Chunks::new();

        chunks.text(self.chunk(|formatter| {
            formatter.write_keyword(Keyword::For);
            formatter.write_space();
            formatter.write_identifier(for_loop.identifier);
            formatter.write_space();
            formatter.write_keyword(Keyword::In);
            formatter.write_space();
        }));

        match for_loop.range {
            ForRange::Range(for_bounds) => {
                self.format_expression(for_bounds.start, &mut chunks);
                chunks.text(self.chunk(|formatter| {
                    formatter.skip_comments_and_whitespace();
                    formatter.write_current_token();
                    formatter.bump();
                }));
                self.format_expression(for_bounds.end, &mut chunks);
            }
            ForRange::Array(expression) => {
                self.format_expression(expression, &mut chunks);
            }
        }

        chunks.text(self.chunk(|formatter| {
            formatter.write_space();
        }));

        let ExpressionKind::Block(block) = for_loop.block.kind else {
            panic!("Expected a block expression for for loop body");
        };

        chunks.group(self.format_block_expression(
            block, true, // force multiple lines
        ));

        // If there's a trailing semicolon, remove it
        chunks.text(self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace();
            if formatter.token == Token::Semicolon {
                formatter.bump();
            }
        }));

        chunks
    }

    fn format_comptime_statement(&mut self, statement: Statement) -> Chunks {
        let mut chunks = Chunks::new();

        // A comptime statement can be a let, a block or a for.
        // We always want to force multiple lines except for let.
        chunks.force_multiple_lines = !matches!(statement.kind, StatementKind::Let(..));

        chunks.text(self.chunk(|formatter| {
            formatter.write_keyword(Keyword::Comptime);
            formatter.write_space();
        }));
        self.format_statement(statement, &mut chunks);
        chunks
    }

    fn format_semi_statement(&mut self, expression: Expression) -> Chunks {
        let mut chunks = Chunks::new();

        self.format_expression(expression, &mut chunks);

        chunks.text(self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace();
            formatter.write_semicolon();
        }));

        chunks
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_format;

    #[test]
    fn format_expression_statement() {
        let src = " fn foo() { 1 } ";
        let expected = "fn foo() {
    1
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_semi_statement() {
        let src = " fn foo() { 1 ; } ";
        let expected = "fn foo() {
    1;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_break_statement() {
        let src = " fn foo() { break  ; } ";
        let expected = "fn foo() {
    break;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_continue_statement() {
        let src = " fn foo() { continue  ; } ";
        let expected = "fn foo() {
    continue;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_let_statement_no_type() {
        let src = " fn foo() { let  x  =  1 ; } ";
        let expected = "fn foo() {
    let x = 1;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_let_statement_with_type() {
        let src = " fn foo() { let  x  :  Field  =  1 ; } ";
        let expected = "fn foo() {
    let x: Field = 1;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_assign() {
        let src = " fn foo() { x  =  2 ; } ";
        let expected = "fn foo() {
    x = 2;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_assign_to_member() {
        let src = " fn foo() { x . y  =  2 ; } ";
        let expected = "fn foo() {
    x.y = 2;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_assign_to_index() {
        let src = " fn foo() { x [ y ]  =  2 ; } ";
        let expected = "fn foo() {
    x[y] = 2;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_assign_to_dereference() {
        let src = " fn foo() { * x  =  2 ; } ";
        let expected = "fn foo() {
    *x = 2;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_assign_with_parentheses() {
        let src = " fn foo() { ( array[0] )[1] = 2; } ";
        let expected = "fn foo() {
    (array[0])[1] = 2;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_op_assign() {
        let src = " fn foo() { x  + =  2 ; } ";
        let expected = "fn foo() {
    x += 2;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_comptime_let_statement() {
        let src = " fn foo() { comptime  let  x  =  1 ; } ";
        let expected = "fn foo() {
    comptime let x = 1;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_block_statement() {
        let src = " fn foo() { { } } ";
        let expected = "fn foo() {
    {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_block_statement_with_inline_block_comment() {
        let src = " fn foo() { { /* hello */ } } ";
        let expected = "fn foo() {
    { /* hello */ }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_block_statement() {
        let src = " fn foo() { { 1 ; 2 } } ";
        let expected = "fn foo() {
    {
        1;
        2
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_unsafe_statement() {
        let src = " fn foo() { unsafe { 1  } } ";
        let expected = "fn foo() {
    unsafe {
        1
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_comptime_statement_one_statement() {
        let src = " fn foo() { comptime { 1  } } ";
        let expected = "fn foo() {
    comptime {
        1
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_comptime_block_statement() {
        let src = " fn foo() { comptime { 1 ; 2 } } ";
        let expected = "fn foo() {
    comptime {
        1;
        2
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_for_array() {
        let src = " fn foo() {  for  x  in  array  {  1  } } ";
        let expected = "fn foo() {
    for x in array {
        1
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_for_array_trailing_semicolon() {
        let src = " fn foo() {  for  x  in  array  {  1  } ; } ";
        let expected = "fn foo() {
    for x in array {
        1
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_for_range_exclusive() {
        let src = " fn foo() {  for  x  in  1 .. 10  {  1  } } ";
        let expected = "fn foo() {
    for x in 1..10 {
        1
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_for_range_inclusive() {
        let src = " fn foo() {  for  x  in  1 ..= 10  {  1  } } ";
        let expected = "fn foo() {
    for x in 1..=10 {
        1
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_assert() {
        let src = r#" fn foo() {  assert ( true , "hello" ) ;  } "#;
        let expected = r#"fn foo() {
    assert(true, "hello");
}
"#;
        assert_format(src, expected);
    }

    #[test]
    fn format_assert_eq() {
        let src = r#" fn foo() {  assert ( 1 , 2 , "hello" ) ;  } "#;
        let expected = r#"fn foo() {
    assert(1, 2, "hello");
}
"#;
        assert_format(src, expected);
    }
}
