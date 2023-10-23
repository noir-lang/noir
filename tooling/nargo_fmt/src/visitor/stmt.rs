use std::iter::zip;

use noirc_frontend::{Statement, StatementKind};

use super::ExpressionType;

impl super::FmtVisitor<'_> {
    pub(crate) fn visit_stmts(&mut self, stmts: Vec<Statement>) {
        let len = stmts.len();

        for (Statement { kind, span }, index) in zip(stmts, 0..) {
            let is_last = index == len;

            match kind {
                StatementKind::Expression(expr) => self.visit_expr(
                    expr,
                    if is_last { ExpressionType::SubExpression } else { ExpressionType::Statement },
                ),
                StatementKind::Semi(expr) => {
                    self.visit_expr(expr, ExpressionType::Statement);
                    self.push_str(";");
                }
                StatementKind::Let(let_stmt) => {
                    let let_str =
                        self.slice(span.start()..let_stmt.expression.span.start()).trim_end();
                    let expr_str =
                        self.format_expr(let_stmt.expression, ExpressionType::SubExpression);

                    self.push_rewrite(format!("{let_str} {expr_str};"), span);
                }
                StatementKind::Error => unreachable!(),
                _ => self.format_missing(span.end()),
            }

            self.last_position = span.end();
        }
    }
}
