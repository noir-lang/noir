use noirc_frontend::{
    ast::{ArrayLiteral, CastExpression, Expression, ExpressionKind, Literal},
    token::{Keyword, Token},
};

use super::{
    chunks::{Chunks, TextChunk},
    Formatter,
};

impl<'a> Formatter<'a> {
    pub(super) fn format_expression(&mut self, expression: Expression, chunks: &mut Chunks) {
        chunks.leading_comment(self.skip_comments_and_whitespace_chunk());

        match expression.kind {
            ExpressionKind::Literal(literal) => self.format_literal(literal, chunks),
            ExpressionKind::Block(_block_expression) => todo!("Format block"),
            ExpressionKind::Prefix(_prefix_expression) => todo!("Format prefix"),
            ExpressionKind::Index(_index_expression) => todo!("Format index"),
            ExpressionKind::Call(_call_expression) => todo!("Format call"),
            ExpressionKind::MethodCall(_method_call_expression) => todo!("Format method call"),
            ExpressionKind::Constructor(_constructor_expression) => todo!("Format constructor"),
            ExpressionKind::MemberAccess(_member_access_expression) => {
                todo!("Format member access")
            }
            ExpressionKind::Cast(cast_expression) => {
                chunks.group(self.format_cast(*cast_expression));
            }
            ExpressionKind::Infix(_infix_expression) => todo!("Format infix"),
            ExpressionKind::If(_if_expression) => todo!("Format if"),
            ExpressionKind::Variable(path) => {
                chunks.text(self.chunk(|formatter| {
                    formatter.format_path(path);
                }));
            }
            ExpressionKind::Tuple(_vec) => todo!("Format tuple"),
            ExpressionKind::Lambda(_lambda) => todo!("Format lambda"),
            ExpressionKind::Parenthesized(_expression) => todo!("Format parenthesized"),
            ExpressionKind::Quote(_tokens) => todo!("Format quote"),
            ExpressionKind::Unquote(_expression) => todo!("Format unquote"),
            ExpressionKind::Comptime(_block_expression, _span) => todo!("Format comptime"),
            ExpressionKind::Unsafe(_block_expression, _span) => todo!("Format unsafe"),
            ExpressionKind::AsTraitPath(_as_trait_path) => todo!("Format as trait path"),
            ExpressionKind::TypePath(_type_path) => todo!("Format type path"),
            ExpressionKind::Resolved(..)
            | ExpressionKind::Interned(..)
            | ExpressionKind::InternedStatement(..)
            | ExpressionKind::Error => unreachable!("Should not be present in the AST"),
        }
    }

    fn format_literal(&mut self, literal: Literal, chunks: &mut Chunks) {
        match literal {
            Literal::Unit => chunks.text(self.chunk(|formatter| {
                formatter.write_left_paren();
                formatter.write_right_paren();
            })),
            Literal::Bool(_)
            | Literal::Integer(..)
            | Literal::Str(_)
            | Literal::FmtStr(_)
            | Literal::RawStr(..) => chunks.text(self.chunk(|formatter| {
                formatter.write_current_token();
                formatter.bump();
            })),
            Literal::Array(array_literal) => chunks.group(self.format_array_literal(
                array_literal,
                false, // is slice
            )),
            Literal::Slice(array_literal) => {
                chunks.group(self.format_array_literal(
                    array_literal,
                    true, // is slice
                ))
            }
        }
    }

    fn format_array_literal(&mut self, literal: ArrayLiteral, is_slice: bool) -> Chunks {
        let mut chunks = Chunks::new().with_multiple_chunks_per_line();

        chunks.text(self.chunk(|formatter| {
            if is_slice {
                formatter.write_token(Token::Ampersand);
            }
            formatter.write_left_bracket();
        }));

        chunks.increase_indentation();
        chunks.line();

        match literal {
            ArrayLiteral::Standard(exprs) => {
                for (index, expr) in exprs.into_iter().enumerate() {
                    if index > 0 {
                        chunks.text(self.chunk(|formatter| {
                            formatter.write_comma();
                        }));
                        chunks.trailing_comment(self.skip_comments_and_whitespace_chunk());
                        chunks.space_or_line();
                    }
                    self.format_expression(expr, &mut chunks)
                }

                let chunk = self.chunk(|formatter| {
                    formatter.skip_comments_and_whitespace();

                    // Trailing comma
                    if formatter.token == Token::Comma {
                        formatter.bump();
                        formatter.skip_comments_and_whitespace();
                    }
                });

                // Make sure to put a trailing comma before the last parameter comments, if there were any
                chunks.text_if_multiline(TextChunk::new(",".to_string()));
                chunks.text(chunk);
            }

            ArrayLiteral::Repeated { repeated_element, length } => {
                self.format_expression(*repeated_element, &mut chunks);
                chunks.text(self.chunk(|formatter| {
                    formatter.write_semicolon();
                    formatter.write_space();
                }));
                self.format_expression(*length, &mut chunks);
            }
        }

        chunks.decrease_indentation();
        chunks.line();

        chunks.text(self.chunk(|formatter| formatter.write_right_bracket()));

        chunks
    }

    fn format_cast(&mut self, cast_expression: CastExpression) -> Chunks {
        let mut chunks = Chunks::new();
        self.format_expression(cast_expression.lhs, &mut chunks);
        chunks.text(self.chunk(|formatter| {
            formatter.write_space();
            formatter.write_keyword(Keyword::As);
            formatter.write_space();
            formatter.format_type(cast_expression.r#type);
        }));
        chunks
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_format, assert_format_with_max_width};

    #[test]
    fn format_unit() {
        let src = "global x =  ( ) ;";
        let expected = "global x = ();\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_false() {
        let src = "global x =  false ;";
        let expected = "global x = false;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_true() {
        let src = "global x =  true ;";
        let expected = "global x = true;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_integer() {
        let src = "global x =  42 ;";
        let expected = "global x = 42;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_string() {
        let src = "global x =  \"hello\" ;";
        let expected = "global x = \"hello\";\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_fmtstr() {
        let src = "global x =  f\"hello\" ;";
        let expected = "global x = f\"hello\";\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_standard_array() {
        let src = "global x = [ 1 , 2 , 3 , ] ;";
        let expected = "global x = [1, 2, 3];\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_standard_slice() {
        let src = "global x = & [ 1 , 2 , 3 , ] ;";
        let expected = "global x = &[1, 2, 3];\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_repeated_array() {
        let src = "global x = [ 1 ; 3 ] ;";
        let expected = "global x = [1; 3];\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_long_array_in_global() {
        let src = "global x = [ 1 , 2 , 3 , 4, 5, ] ;";
        let expected = "global x =
    [1, 2, 3, 4, 5];
";
        assert_format_with_max_width(src, expected, 20);
    }

    #[test]
    fn format_long_array_in_global_2() {
        let src = "global x = [ 1 , 2 , 3 , 4, 5, ] ;

global y = 1;
        ";
        let expected = "global x =
    [1, 2, 3, 4, 5];

global y = 1;
";
        assert_format_with_max_width(src, expected, 20);
    }

    #[test]
    fn format_very_long_array_in_global() {
        let src = "global x = [ 1 , 2 , 3 , 4, 5, 6, 789, 123, 234, 345] ;";
        let expected = "global x =
    [
        1, 2, 3, 4, 5, 6,
        789, 123, 234,
        345,
    ];
";
        assert_format_with_max_width(src, expected, 25);
    }

    #[test]
    fn format_array_in_global_with_line_comments() {
        let src = "global x = [ // hello
        1 , 2 ] ;";
        let expected = "global x =
    [
        // hello
        1, 2,
    ];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_line_comments_2() {
        let src = "global x = [ // hello
         [ 1 , 2 ]  ] ;";
        let expected = "global x =
    [
        // hello
        [1, 2],
    ];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_line_comments_3() {
        let src = "global x =
    [ 
        // hello
        [1, 2],  
    ];
";
        let expected = "global x =
    [
        // hello
        [1, 2],
    ];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_line_comments_4() {
        let src = "global x =
    [
        1, // world 
        2, 3,
    ];
";
        let expected = "global x =
    [
        1, // world
        2, 3,
    ];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_block_comments() {
        let src = "global x = [ /* hello */
        1 , 2 ] ;";
        let expected = "global x =
    [
        /* hello */
        1, 2,
    ];
";
        assert_format_with_max_width(src, expected, 20);
    }

    #[test]
    fn format_cast() {
        let src = "global x =  1  as  u8 ;";
        let expected = "global x = 1 as u8;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_variable() {
        let src = "global x =  y ;";
        let expected = "global x = y;\n";
        assert_format(src, expected);
    }
}
