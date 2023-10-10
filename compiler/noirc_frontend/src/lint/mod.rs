mod deprecated;

use fm::FileId;
use noirc_errors::{CustomDiagnostic, Span};

use crate::{
    hir::def_collector::dc_crate::CompilationError,
    hir_def::{expr::HirExpression, function::HirFunction, stmt::HirStatement},
    node_interner::{ExprId, FuncId, NodeInterner, StmtId},
};

#[derive(Debug, Clone)]
pub enum LintDiagnostic {
    Deprecated { name: String, note: Option<String>, span: Span },
}

impl From<LintDiagnostic> for noirc_errors::CustomDiagnostic {
    fn from(value: LintDiagnostic) -> Self {
        match value {
            LintDiagnostic::Deprecated { name, note, span } => {
                let primary_message = format!("use of deprecated function {name}");
                let secondary_message = note.unwrap_or_default();

                CustomDiagnostic::simple_warning(primary_message, secondary_message, span)
            }
        }
    }
}

pub(crate) fn functions(
    interner: &NodeInterner,
    func_ids: &[(FileId, FuncId)],
) -> Vec<(CompilationError, fm::FileId)> {
    func_ids
        .iter()
        .flat_map(|(file_id, func_id)| {
            function(interner, *func_id).into_iter().map(|diagnostic| (diagnostic.into(), *file_id))
        })
        .collect()
}

fn function(interner: &NodeInterner, func_id: FuncId) -> Vec<LintDiagnostic> {
    let mut out = Vec::new();
    let lints: [&mut dyn Lint; 1] =
        [&mut deprecated::Deprecated { interner, diagnostics: Vec::new() }];

    for lint in lints {
        let mut visitor = Visitor { interner, lint };
        visitor.visit_func(func_id);

        out.extend(visitor.lint.finish());
    }

    out
}

struct Visitor<'me> {
    interner: &'me NodeInterner,
    lint: &'me mut dyn Lint,
}

impl<'me> Visitor<'me> {
    fn visit_func(&mut self, func_id: FuncId) {
        let func = self.interner.function(&func_id);
        let body = *func.as_expr();

        self.lint.check_func(func_id);
        self.lint.check_func_body(func);

        self.visit_expr(body);
    }

    fn visit_expr(&mut self, expr_id: ExprId) {
        let expr = self.interner.expression(&expr_id);
        self.lint.check_expr(&expr);

        match expr {
            HirExpression::Ident(_) | HirExpression::Literal(_) => {}
            HirExpression::Block(block) => {
                for stmt_id in block.0 {
                    self.visit_stmt(stmt_id);
                }
            }
            HirExpression::Prefix(prefix) => {
                self.visit_expr(prefix.rhs);
            }
            HirExpression::Infix(infix) => {
                self.visit_expr(infix.lhs);
                self.visit_expr(infix.rhs);
            }
            HirExpression::Index(index) => {
                self.visit_expr(index.collection);
                self.visit_expr(index.index);
            }
            HirExpression::Constructor(constructor) => {
                for (_field_name, field) in constructor.fields {
                    self.visit_expr(field);
                }
            }
            HirExpression::MemberAccess(member_access) => {
                self.visit_expr(member_access.lhs);
            }
            HirExpression::Call(call) => {
                self.visit_expr(call.func);

                for arg in call.arguments {
                    self.visit_expr(arg);
                }
            }
            HirExpression::MethodCall(_) => todo!(),
            HirExpression::Cast(cast) => {
                self.visit_expr(cast.lhs);
            }
            HirExpression::If(if_expr) => {
                self.visit_expr(if_expr.condition);
                self.visit_expr(if_expr.consequence);

                if let Some(alternative) = if_expr.alternative {
                    self.visit_expr(alternative);
                }
            }
            HirExpression::Tuple(tuple) => {
                for expr in tuple {
                    self.visit_expr(expr);
                }
            }
            HirExpression::Lambda(_) => todo!(),
            HirExpression::TraitMethodReference(_, _) => todo!(),
            HirExpression::Error => todo!(),
        }
    }

    fn visit_stmt(&mut self, stmt_id: StmtId) {
        match self.interner.statement(&stmt_id) {
            HirStatement::Let(let_stmt) => {
                self.visit_expr(let_stmt.expression);
            }
            HirStatement::Constrain(constrain) => {
                self.visit_expr(constrain.0);
            }
            HirStatement::Assign(assign) => {
                self.visit_expr(assign.expression);
            }
            HirStatement::For(for_stmt) => {
                self.visit_expr(for_stmt.start_range);
                self.visit_expr(for_stmt.end_range);
                self.visit_expr(for_stmt.block);
            }
            HirStatement::Expression(expr) | HirStatement::Semi(expr) => self.visit_expr(expr),
            HirStatement::Error => {}
        }
    }
}

trait Lint {
    fn check_func(&mut self, _func: FuncId) {}
    fn check_func_body(&mut self, _func: HirFunction) {}
    fn check_expr(&mut self, _expr: &HirExpression) {}
    fn finish(&mut self) -> Vec<LintDiagnostic>;
}
