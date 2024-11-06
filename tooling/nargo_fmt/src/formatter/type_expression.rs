use noirc_frontend::{ast::UnresolvedTypeExpression, token::Token};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_type_expression(&mut self, type_expr: UnresolvedTypeExpression) {
        self.skip_comments_and_whitespace();

        // Parenthesized type expressions exist but are not represented in the AST
        while let Token::LeftParen = self.token {
            self.write_left_paren();
        }

        match type_expr {
            UnresolvedTypeExpression::Variable(path) => self.format_path(path),
            UnresolvedTypeExpression::Constant(..) => {
                self.write_current_token_and_bump();
            }
            UnresolvedTypeExpression::BinaryOperation(lhs, _operator, rhs, _span) => {
                self.format_type_expression(*lhs);
                self.write_space();
                self.write_current_token_and_bump();
                self.write_space();
                self.format_type_expression(*rhs);
            }
            UnresolvedTypeExpression::AsTraitPath(..) => {
                unreachable!("Should not be present in the AST")
            }
        }

        self.skip_comments_and_whitespace();

        // Parenthesized type expressions exist but are not represented in the AST
        while let Token::RightParen = self.token {
            self.write_right_paren();
        }
    }
}
