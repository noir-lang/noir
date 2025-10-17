use crate::{
    ast::{ExpressionKind, Literal, Pattern},
    hir::{def_collector::dc_crate::UnresolvedGlobal, resolution::errors::ResolverError},
    hir_def::stmt::HirStatement,
    node_interner::{DependencyId, GlobalId, GlobalValue},
    token::SecondaryAttributeKind,
};

use super::Elaborator;

impl Elaborator<'_> {
    pub(super) fn elaborate_global(&mut self, global: UnresolvedGlobal) {
        let old_module = std::mem::replace(&mut self.local_module, global.module_id);
        let old_item = self.current_item.take();

        let global_id = global.global_id;
        self.current_item = Some(DependencyId::Global(global_id));
        let let_stmt = global.stmt_def;

        let name = if self.interner.is_in_lsp_mode() {
            Some(let_stmt.pattern.name_ident().to_string())
        } else {
            None
        };

        let location = let_stmt.pattern.location();

        if !self.in_contract() {
            for attr in &let_stmt.attributes {
                if matches!(attr.kind, SecondaryAttributeKind::Abi(_)) {
                    self.push_err(ResolverError::AbiAttributeOutsideContract {
                        location: attr.location,
                    });
                }
            }
        }

        if !let_stmt.comptime && matches!(let_stmt.pattern, Pattern::Mutable(..)) {
            self.push_err(ResolverError::MutableGlobal { location });
        }

        let (let_statement, _typ) = self
            .elaborate_in_comptime_context(|this| this.elaborate_let(let_stmt, Some(global_id)));

        if let_statement.r#type.contains_reference() {
            self.push_err(ResolverError::ReferencesNotAllowedInGlobals { location });
        }

        let let_statement = HirStatement::Let(let_statement);

        let statement_id = self.interner.get_global(global_id).let_statement;
        self.interner.replace_statement(statement_id, let_statement);

        self.elaborate_comptime_global(global_id);

        if let Some(name) = name {
            self.interner.register_global(global_id, name, location, global.visibility);
        }

        self.local_module = old_module;
        self.current_item = old_item;
    }

    pub(super) fn elaborate_comptime_global(&mut self, global_id: GlobalId) {
        let let_statement = self
            .interner
            .get_global_let_statement(global_id)
            .expect("Let statement of global should be set by elaborate_global_let");

        let global = self.interner.get_global(global_id);
        let definition_id = global.definition_id;
        let location = global.location;
        let mut interpreter = self.setup_interpreter();

        if let Err(error) = interpreter.evaluate_let(let_statement) {
            self.push_err(error);
        } else {
            let value = interpreter
                .lookup_id(definition_id, location)
                .expect("The global should be defined since evaluate_let did not error");

            self.debug_comptime(location, |interner| value.display(interner).to_string());

            self.interner.get_global_mut(global_id).value = GlobalValue::Resolved(value);
        }
    }

    /// If the given global is unresolved, elaborate it and return true
    pub(super) fn elaborate_global_if_unresolved(&mut self, global_id: &GlobalId) -> bool {
        if let Some(global) = self.unresolved_globals.remove(global_id) {
            self.elaborate_global(global);
            true
        } else {
            false
        }
    }
}

/// Separate the globals Vec into two. The first element in the tuple will be the
/// literal globals, except for arrays, and the second will be all other globals.
/// We exclude array literals as they can contain complex types
pub(super) fn filter_literal_globals(
    globals: Vec<UnresolvedGlobal>,
) -> (Vec<UnresolvedGlobal>, Vec<UnresolvedGlobal>) {
    globals.into_iter().partition(|global| match &global.stmt_def.expression.kind {
        ExpressionKind::Literal(literal) => !matches!(literal, Literal::Array(_)),
        _ => false,
    })
}
