use noirc_frontend::{
    ast::{ExpressionKind, Statement, StatementKind, UnresolvedTypeData},
    token::{Keyword, Token},
};

use super::{chunks::Chunks, Formatter};

impl<'a> Formatter<'a> {
    pub(super) fn format_statement(&mut self, statement: Statement, mut chunks: &mut Chunks) {
        chunks.leading_comment(self.skip_comments_and_whitespace_chunk());

        match statement.kind {
            StatementKind::Let(let_statement) => {
                let mut group = Chunks::new();

                group.text(self.chunk(|formatter| {
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
                group.increase_indentation();
                group.space_or_line();
                self.format_expression(let_statement.expression, &mut group);
                group.text(self.chunk(|formatter| {
                    formatter.write_semicolon();
                }));
                group.decrease_indentation();

                chunks.group(group);
            }
            StatementKind::Constrain(_constrain_statement) => todo!("Format constrain statement"),
            StatementKind::Expression(expression) => {
                self.format_expression(expression, &mut chunks);
            }
            StatementKind::Assign(assign_statement) => {
                let mut group = Chunks::new();
                let mut is_op_assign = false;

                group.text(self.chunk(|formatter| {
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
                group.increase_indentation();
                group.space_or_line();

                if is_op_assign {
                    let ExpressionKind::Infix(infix) = assign_statement.expression.kind else {
                        panic!("Expected an infix expression for op assign");
                    };
                    self.format_expression(infix.rhs, &mut group);
                } else {
                    self.format_expression(assign_statement.expression, &mut group);
                }
                group.text(self.chunk(|formatter| {
                    formatter.write_semicolon();
                }));
                group.decrease_indentation();

                chunks.group(group);
            }
            StatementKind::For(_for_loop_statement) => todo!("Format for loop statement"),
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
            StatementKind::Comptime(_statement) => todo!("Format comptime statement"),
            StatementKind::Semi(expression) => {
                self.format_expression(expression, &mut chunks);

                chunks.text(self.chunk(|formatter| {
                    formatter.skip_comments_and_whitespace();
                    formatter.write_semicolon();
                }));
            }
            StatementKind::Interned(..) | StatementKind::Error => {
                unreachable!("Should not be present in the AST")
            }
        }
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
}
