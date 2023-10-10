use super::{Lint, LintDiagnostic};
use crate::hir_def::expr::{HirExpression, HirIdent};
use crate::node_interner::{DefinitionKind, NodeInterner};

pub(crate) struct Deprecated<'me> {
    pub(crate) interner: &'me NodeInterner,
    pub(crate) diagnostics: Vec<LintDiagnostic>,
}

impl Lint for Deprecated<'_> {
    fn check_expr(&mut self, expr: &HirExpression) {
        if let HirExpression::Ident(HirIdent { location, id }) = expr {
            if let Some(DefinitionKind::Function(func_id)) =
                self.interner.try_definition(*id).map(|def| &def.kind)
            {
                let attributes = self.interner.function_attributes(func_id);
                if let Some(note) = attributes.get_deprecated_note() {
                    self.diagnostics.push(LintDiagnostic::Deprecated {
                        name: self.interner.definition_name(*id).to_string(),
                        note,
                        span: location.span,
                    });
                }
            }
        }
    }

    fn finish(&mut self) -> Vec<LintDiagnostic> {
        std::mem::take(&mut self.diagnostics)
    }
}
