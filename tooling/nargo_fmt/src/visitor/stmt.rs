use noirc_frontend::{Statement, StatementKind};

impl super::FmtVisitor<'_> {
    pub(crate) fn visit_stmts(&mut self, stmts: Vec<Statement>) {
        for Statement { kind, span } in stmts {
            match kind {
                StatementKind::Expression(expr) => self.visit_expr(expr),
                StatementKind::Semi(expr) => {
                    self.visit_expr(expr);
                    self.push_str(";");
                }
                StatementKind::Let(let_stmt) => {
                    let let_str =
                        slice!(self, span.start(), let_stmt.expression.span.start()).trim_end();
                    let expr_str = self.format_expr(let_stmt.expression);

                    self.push_rewrite(format!("{let_str} {expr_str};"), span);
                }
                StatementKind::Error => unreachable!(),
                _ => self.format_missing(span.end()),
            }

            self.last_position = span.end();
        }
    }
}
