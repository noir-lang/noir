use noirc_frontend::{
    ast::{
        AssignStatement, ConstrainKind, ConstrainStatement, Expression, ExpressionKind,
        ForLoopStatement, ForRange, LetStatement, Pattern, Statement, StatementKind,
        UnresolvedType, UnresolvedTypeData,
    },
    token::{Keyword, SecondaryAttribute, Token, TokenKind},
};

use crate::chunks::{ChunkFormatter, ChunkGroup, GroupKind};

impl<'a, 'b> ChunkFormatter<'a, 'b> {
    pub(super) fn format_statement(
        &mut self,
        statement: Statement,
        group: &mut ChunkGroup,
        mut ignore_next: bool,
    ) {
        // First skip any whitespace to avoid writing multiple lines
        group.text(self.chunk(|formatter| {
            formatter.skip_whitespace();
        }));

        // Now write any leading comment respecting multiple newlines after them
        group.leading_comment(self.chunk(|formatter| {
            // Doc comments for a let statement could come before a potential non-doc comment
            if formatter.token.kind() == TokenKind::OuterDocComment {
                formatter.format_outer_doc_comments();
            }

            formatter.skip_comments_and_whitespace_writing_multiple_lines_if_found();

            // Or doc comments could come after a potential non-doc comment
            if formatter.token.kind() == TokenKind::OuterDocComment {
                formatter.format_outer_doc_comments();
            }
        }));

        ignore_next |= self.ignore_next;

        if ignore_next {
            group.text(self.chunk(|formatter| {
                formatter.write_and_skip_span_without_formatting(statement.span);
            }));
            return;
        }

        match statement.kind {
            StatementKind::Let(let_statement) => {
                group.group(self.format_let_statement(let_statement));
            }
            StatementKind::Constrain(constrain_statement) => {
                group.group(self.format_constrain_statement(constrain_statement));
            }
            StatementKind::Expression(expression) => match expression.kind {
                ExpressionKind::Block(block) => group.group(self.format_block_expression(
                    block, true, // force multiple lines
                )),
                ExpressionKind::Unsafe(block, _) => {
                    group.group(self.format_unsafe_expression(
                        block, true, // force multiple lines
                    ));
                }
                ExpressionKind::If(if_expression) => {
                    group.group(self.format_if_expression(
                        *if_expression,
                        true, // force multiple lines
                    ));
                }
                _ => self.format_expression(expression, group),
            },
            StatementKind::Assign(assign_statement) => {
                group.group(self.format_assign(assign_statement));
            }
            StatementKind::For(for_loop_statement) => {
                group.group(self.format_for_loop(for_loop_statement));
            }
            StatementKind::Loop(block) => {
                group.group(self.format_loop(block));
            }
            StatementKind::Break => {
                group.text(self.chunk(|formatter| {
                    formatter.write_keyword(Keyword::Break);
                    formatter.write_semicolon();
                }));
            }
            StatementKind::Continue => {
                group.text(self.chunk(|formatter| {
                    formatter.write_keyword(Keyword::Continue);
                    formatter.write_semicolon();
                }));
            }
            StatementKind::Comptime(statement) => {
                group.group(self.format_comptime_statement(*statement));
            }
            StatementKind::Semi(expression) => {
                group.group(self.format_semi_statement(expression));
            }
            StatementKind::Interned(..) | StatementKind::Error => {
                unreachable!("Should not be present in the AST")
            }
        }
    }

    fn format_let_statement(&mut self, let_statement: LetStatement) -> ChunkGroup {
        self.format_let_or_global(
            Keyword::Let,
            let_statement.pattern,
            let_statement.r#type,
            Some(let_statement.expression),
            let_statement.attributes,
        )
    }

    pub(super) fn format_let_or_global(
        &mut self,
        keyword: Keyword,
        pattern: Pattern,
        typ: UnresolvedType,
        value: Option<Expression>,
        attributes: Vec<SecondaryAttribute>,
    ) -> ChunkGroup {
        let mut group = ChunkGroup::new();

        group.text(self.chunk(|formatter| {
            formatter.format_secondary_attributes(attributes);
            formatter.write_keyword(keyword);
            formatter.write_space();
            formatter.format_pattern(pattern);
            if typ.typ != UnresolvedTypeData::Unspecified {
                formatter.write_token(Token::Colon);
                formatter.write_space();
                formatter.format_type(typ);
            }
        }));

        if let Some(value) = value {
            group.text(self.chunk(|formatter| {
                formatter.write_space();
                formatter.write_token(Token::Assign);
                formatter.write_space();
            }));

            let mut value_group = ChunkGroup::new();
            value_group.kind = GroupKind::AssignValue;
            self.format_expression(value, &mut value_group);
            value_group.semicolon(self);
            group.group(value_group);
        } else {
            group.semicolon(self);
        }

        group
    }

    fn format_constrain_statement(
        &mut self,
        constrain_statement: ConstrainStatement,
    ) -> ChunkGroup {
        let mut group = ChunkGroup::new();

        let keyword = match constrain_statement.kind {
            ConstrainKind::Assert => Keyword::Assert,
            ConstrainKind::AssertEq => Keyword::AssertEq,
            ConstrainKind::Constrain => {
                unreachable!("constrain always produces an error, and the formatter doesn't run when there are errors")
            }
        };

        group.text(self.chunk(|formatter| {
            formatter.write_keyword(keyword);
            formatter.write_left_paren();
        }));

        group.kind = GroupKind::ExpressionList {
            prefix_width: group.width(),
            expressions_count: constrain_statement.arguments.len(),
        };

        self.format_expressions_separated_by_comma(
            constrain_statement.arguments,
            false, // force trailing comma
            &mut group,
        );

        group.text(self.chunk(|formatter| {
            formatter.write_right_paren();
            formatter.write_semicolon();
        }));

        group
    }

    fn format_assign(&mut self, assign_statement: AssignStatement) -> ChunkGroup {
        let mut group = ChunkGroup::new();
        let mut is_op_assign = false;

        group.text(self.chunk(|formatter| {
            formatter.format_lvalue(assign_statement.lvalue);
            formatter.write_space();
            if formatter.is_at(Token::Assign) {
                formatter.write_token(Token::Assign);
            } else {
                // This is something like `x += 1`, which is parsed as an
                // Assign with an InfixExpression as its right-hand side: `x = x + 1`.
                // There will always be two tokens here, like `+ =` or `> >=`.
                formatter.write_current_token();
                formatter.bump();
                formatter.skip_comments_and_whitespace();
                formatter.write_current_token();
                formatter.bump();

                is_op_assign = true;
            }
            formatter.write_space();
        }));

        let mut value_group = ChunkGroup::new();
        value_group.kind = GroupKind::AssignValue;

        if is_op_assign {
            let ExpressionKind::Infix(infix) = assign_statement.expression.kind else {
                panic!("Expected an infix expression for op assign");
            };
            self.format_expression(infix.rhs, &mut value_group);
        } else {
            self.format_expression(assign_statement.expression, &mut value_group);
        }
        value_group.semicolon(self);
        group.group(value_group);

        group
    }

    fn format_for_loop(&mut self, for_loop: ForLoopStatement) -> ChunkGroup {
        let mut group = ChunkGroup::new();

        group.text(self.chunk(|formatter| {
            formatter.write_keyword(Keyword::For);
            formatter.write_space();
            formatter.write_identifier(for_loop.identifier);
            formatter.write_space();
            formatter.write_keyword(Keyword::In);
            formatter.write_space();
        }));

        match for_loop.range {
            ForRange::Range(for_bounds) => {
                self.format_expression(for_bounds.start, &mut group);
                group.text(self.chunk(|formatter| {
                    formatter.skip_comments_and_whitespace();
                    formatter.write_current_token();
                    formatter.bump();
                }));
                self.format_expression(for_bounds.end, &mut group);
            }
            ForRange::Array(expression) => {
                self.format_expression(expression, &mut group);
            }
        }

        group.space(self);

        let ExpressionKind::Block(block) = for_loop.block.kind else {
            panic!("Expected a block expression for for loop body");
        };

        group.group(self.format_block_expression(
            block, true, // force multiple lines
        ));

        // If there's a trailing semicolon, remove it
        group.text(self.chunk(|formatter| {
            formatter.skip_whitespace_if_it_is_not_a_newline();
            if formatter.is_at(Token::Semicolon) {
                formatter.bump();
            }
        }));

        group
    }

    fn format_loop(&mut self, block: Expression) -> ChunkGroup {
        let mut group = ChunkGroup::new();

        group.text(self.chunk(|formatter| {
            formatter.write_keyword(Keyword::Loop);
        }));

        group.space(self);

        let ExpressionKind::Block(block) = block.kind else {
            panic!("Expected a block expression for loop body");
        };

        group.group(self.format_block_expression(
            block, true, // force multiple lines
        ));

        // If there's a trailing semicolon, remove it
        group.text(self.chunk(|formatter| {
            formatter.skip_whitespace_if_it_is_not_a_newline();
            if formatter.is_at(Token::Semicolon) {
                formatter.bump();
            }
        }));

        group
    }

    fn format_comptime_statement(&mut self, statement: Statement) -> ChunkGroup {
        let mut group = ChunkGroup::new();

        // A comptime statement can be a let, a block or a for.
        // We always want to force multiple lines except for let.
        group.force_multiple_lines = !matches!(statement.kind, StatementKind::Let(..));

        group.text(self.chunk(|formatter| {
            formatter.write_keyword(Keyword::Comptime);
            formatter.write_space();
        }));
        self.format_statement(
            statement, &mut group, false, // ignore next
        );
        group
    }

    fn format_semi_statement(&mut self, expression: Expression) -> ChunkGroup {
        let mut group = ChunkGroup::new();

        self.format_expression(expression, &mut group);

        group.text(self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace();
        }));

        group.semicolon(self);

        group
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_format, assert_format_with_max_width};

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
    fn format_let_statement_with_attribute() {
        let src = " fn foo() {   #[allow(unused_variables)] let  x  =  1 ; } ";
        let expected = "fn foo() {
    #[allow(unused_variables)]
    let x = 1;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_let_statement_with_unsafe() {
        let src = " fn foo() { 
        /// Safety: some doc
        let  x  =  unsafe { 1 } ; } ";
        let expected = "fn foo() {
    /// Safety: some doc
    let x = unsafe { 1 };
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_let_statement_with_unsafe_and_comment_before_it() {
        let src = " fn foo() { 
        // Some comment
        /// Safety: some doc
        let  x  =  unsafe { 1 } ; } ";
        let expected = "fn foo() {
    // Some comment
    /// Safety: some doc
    let x = unsafe { 1 };
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
    fn format_assign_to_tuple_member() {
        let src = " fn foo() { x . 0  =  2 ; } ";
        let expected = "fn foo() {
    x.0 = 2;
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
    fn format_shift_right_assign() {
        let src = " fn foo() { x  >>=  2 ; } ";
        let expected = "fn foo() {
    x >>= 2;
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
        let src = " fn foo() { unsafe { 
        1  } } ";
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
    fn format_two_for_separated_by_multiple_lines() {
        let src = " fn foo() {  for  x  in  array  {  1  } 
        
        for  x  in  array  {  1  }
        
        } ";
        let expected = "fn foo() {
    for x in array {
        1
    }

    for x in array {
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

    #[test]
    fn format_if_statement() {
        let src = r#" fn foo() {  if 1 {  2  }  } "#;
        let expected = r#"fn foo() {
    if 1 {
        2
    }
}
"#;
        assert_format(src, expected);
    }

    #[test]
    fn does_not_format_statement_if_there_is_a_directive_not_to() {
        let src = "fn foo() {
    // noir-fmt:ignore
    let  x  =
                  1  ;

    let  y  =
                  2 ;

    // noir-fmt:ignore
    let  z  =
                  3  ;
}\n";
        let expected = "fn foo() {
    // noir-fmt:ignore
    let  x  =
                  1  ;

    let y = 2;

    // noir-fmt:ignore
    let  z  =
                  3  ;
}\n";
        assert_format(src, expected);
    }

    #[test]
    fn attaches_semicolon_to_last_group_in_let_statement() {
        let src = "fn foo() {
    let x = foo(1, 2);
}
";
        let expected = "fn foo() {
    let x =
        foo(1, 2);
}
";
        assert_format_with_max_width(src, expected, "    let x = foo(1, 2);".len() - 1);
    }

    #[test]
    fn attaches_semicolon_to_last_group_in_semi_statement() {
        let src = "fn foo() {
    foo(1, 2, 3, 4, 5);
}
";
        let expected = "fn foo() {
    foo(
        1,
        2,
        3,
        4,
        5,
    );
}
";
        assert_format_with_max_width(src, expected, "    foo(1, 2, 3, 4, 5);".len() - 1);
    }

    #[test]
    fn attaches_semicolon_to_last_group_in_assign() {
        let src = "fn foo() {
    a_long_variable = foo(1, 2);
}
";
        let expected = "fn foo() {
    a_long_variable =
        foo(1, 2);
}
";
        assert_format_with_max_width(src, expected, "    a_long_variable = foo(1, 2);".len() - 1);
    }

    #[test]
    fn long_let_preceded_by_two_newlines() {
        let src = "fn foo() {
    let y = 0;

    let x = 123456;
}
";
        let expected = src;
        assert_format_with_max_width(src, expected, "    let x = 123456;".len());
    }

    #[test]
    fn format_empty_loop() {
        let src = " fn foo() {  loop  {   }  } ";
        let expected = "fn foo() {
    loop {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_non_empty_loop() {
        let src = " fn foo() {  loop  { 1 ; 2  }  } ";
        let expected = "fn foo() {
    loop {
        1;
        2
    }
}
";
        assert_format(src, expected);
    }
}
