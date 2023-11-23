use std::iter::zip;

use noirc_frontend::{
    ConstrainKind, ConstrainStatement, ExpressionKind, ForRange, Statement, StatementKind,
};

use crate::rewrite;

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

                    let expr_str = rewrite::subexpr(self, let_stmt.expression, self.shape());

                    self.push_rewrite(format!("{let_str} {expr_str};"), span);
                }
                StatementKind::Constrain(ConstrainStatement(expr, message, kind)) => {
                    let message =
                        message.map_or(String::new(), |message| format!(", \"{message}\""));
                    let constrain = match kind {
                        ConstrainKind::Assert => {
                            let assertion = rewrite::subexpr(self, expr, self.shape());

                            format!("assert({assertion}{message});")
                        }
                        ConstrainKind::AssertEq => {
                            if let ExpressionKind::Infix(infix) = expr.kind {
                                let lhs = rewrite::subexpr(self, infix.lhs, self.shape());
                                let rhs = rewrite::subexpr(self, infix.rhs, self.shape());

                                format!("assert_eq({lhs}, {rhs}{message});")
                            } else {
                                unreachable!()
                            }
                        }
                        ConstrainKind::Constrain => {
                            let expr = rewrite::subexpr(self, expr, self.shape());
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
                            rewrite::subexpr(self, start, self.shape()),
                            rewrite::subexpr(self, end, self.shape())
                        ),
                        ForRange::Array(array) => rewrite::subexpr(self, array, self.shape()),
                    };
                    let block = rewrite::subexpr(self, for_stmt.block, self.shape());

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
