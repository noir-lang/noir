use std::iter::zip;

use noirc_frontend::{
    ConstrainKind, ConstrainStatement, ExpressionKind, ForRange, Statement, StatementKind,
};

use super::ExpressionType;

impl super::FmtVisitor<'_> {
    pub(crate) fn visit_stmts(&mut self, stmts: Vec<Statement>) {
        let len = stmts.len();

        for (Statement { kind, span }, index) in zip(stmts, 1..) {
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
                StatementKind::Constrain(ConstrainStatement(expr, message, kind)) => {
                    let message =
                        message.map_or(String::new(), |message| format!(", \"{message}\""));
                    let constrain = match kind {
                        ConstrainKind::Assert => {
                            let assertion = self.format_sub_expr(expr);

                            format!("assert({assertion}{message});")
                        }
                        ConstrainKind::AssertEq => {
                            if let ExpressionKind::Infix(infix) = expr.kind {
                                let lhs = self.format_sub_expr(infix.lhs);
                                let rhs = self.format_sub_expr(infix.rhs);

                                format!("assert_eq({lhs}, {rhs}{message});")
                            } else {
                                unreachable!()
                            }
                        }
                        ConstrainKind::Constrain => {
                            let expr = self.format_sub_expr(expr);
                            format!("constrain {expr};")
                        }
                    };

                    self.push_rewrite(constrain, span);
                }
                StatementKind::For(for_stmt) => {
                    let identifier = self.slice(for_stmt.identifier.span());
                    let range = match for_stmt.range {
                        ForRange::Range(start, end) => format!(
                            "{}..{}",
                            self.format_sub_expr(start),
                            self.format_sub_expr(end)
                        ),
                        ForRange::Array(array) => self.format_sub_expr(array),
                    };
                    let block = self.format_sub_expr(for_stmt.block);

                    let result = format!("for {identifier} in {range} {block}");
                    self.push_rewrite(result, span);
                }
                StatementKind::Assign(_) => {
                    self.push_rewrite(self.slice(span).to_string(), span);
                }
                StatementKind::Error => unreachable!(),
            }

            self.last_position = span.end();
        }
    }
}
