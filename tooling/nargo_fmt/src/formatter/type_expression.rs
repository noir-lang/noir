use noirc_frontend::ast::UnresolvedTypeExpression;

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_type_expression(&mut self, type_expr: UnresolvedTypeExpression) {
        self.skip_comments_and_whitespace();

        match type_expr {
            UnresolvedTypeExpression::Variable(path) => self.format_path(path),
            UnresolvedTypeExpression::Constant(..) => {
                self.write_current_token();
                self.bump();
            }
            UnresolvedTypeExpression::BinaryOperation(
                _unresolved_type_expression,
                _binary_type_operator,
                _unresolved_type_expression1,
                _span,
            ) => todo!("Format type expr binary"),
            UnresolvedTypeExpression::AsTraitPath(_as_trait_path) => {
                todo!("Format type expr as trait path")
            }
        }
    }
}
