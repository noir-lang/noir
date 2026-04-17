use noirc_frontend::{ast::LValue, token::Token};

use crate::chunks::{ChunkFormatter, ChunkGroup};

impl ChunkFormatter<'_, '_> {
    pub(super) fn format_lvalue(&mut self, lvalue: LValue, group: &mut ChunkGroup) {
        // Parenthesized l-values exist but are not represented in the AST
        while let Token::LeftParen = self.token {
            group.text(self.chunk(|formatter| {
                formatter.write_left_paren();
            }));
        }

        match lvalue {
            LValue::Path(path) => {
                group.text(self.chunk(|formatter| {
                    formatter.format_path(path);
                }));
            }
            LValue::MemberAccess { object, field_name, location: _ } => {
                self.format_lvalue(*object, group);
                group.text(self.chunk(|formatter| {
                    formatter.write_token(Token::Dot);
                    formatter.write_identifier_or_integer(field_name);
                }));
            }
            LValue::Index { array, index, location: _ } => {
                self.format_lvalue(*array, group);
                group.text(self.chunk(|formatter| {
                    formatter.write_left_bracket();
                }));

                // Keep index comment/newline behavior consistent with rvalue index expressions.
                let comments_chunk = self.skip_comments_and_whitespace_chunk();
                let comments_chunk_has_newlines = comments_chunk.has_newlines;

                if comments_chunk_has_newlines {
                    group.increase_indentation();
                    group.line();
                }

                group.leading_comment(comments_chunk);

                let mut index_group = ChunkGroup::new();
                self.format_expression(index, &mut index_group);
                group.group(index_group);

                if comments_chunk_has_newlines {
                    group.decrease_indentation();
                    group.line();
                }

                group.text(self.chunk(|formatter| {
                    formatter.write_right_bracket();
                }));
            }
            LValue::Dereference(lvalue, _span) => {
                group.text(self.chunk(|formatter| {
                    formatter.write_token(Token::Star);
                }));
                self.format_lvalue(*lvalue, group);
            }
            LValue::Interned(..) => {
                unreachable!("Should not be present in the AST")
            }
        }

        group.text(self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace();
        }));

        // Parenthesized l-values exist but are not represented in the AST
        while let Token::RightParen = self.token {
            group.text(self.chunk(|formatter| {
                formatter.write_right_paren();
            }));
        }
    }
}
