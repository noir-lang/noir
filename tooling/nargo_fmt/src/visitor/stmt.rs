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
                StatementKind::Error => unreachable!(),
                _ => self.format_missing(span.end()),
            }

            self.last_position = span.end();
        }
    }
}
