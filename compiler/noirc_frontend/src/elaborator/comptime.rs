use std::mem::replace;

use crate::{
    hir_def::expr::HirIdent,
    macros_api::Expression,
    node_interner::{DependencyId, ExprId, FuncId},
};

use super::{Elaborator, FunctionContext, ResolverMeta};

impl<'context> Elaborator<'context> {
    /// Elaborate an expression from the middle of a comptime scope.
    /// When this happens we require additional information to know
    /// what variables should be in scope.
    pub fn elaborate_expression_from_comptime(
        &mut self,
        expr: Expression,
        function: Option<FuncId>,
    ) -> ExprId {
        self.function_context.push(FunctionContext::default());
        let old_scope = self.scopes.end_function();
        self.scopes.start_function();
        let function_id = function.map(DependencyId::Function);
        let old_item = replace(&mut self.current_item, function_id);

        // Note: recover_generics isn't good enough here because any existing generics
        // should not be in scope of this new function
        let old_generics = std::mem::take(&mut self.generics);

        let old_crate_and_module = function.map(|function| {
            let meta = self.interner.function_meta(&function);
            let old_crate = replace(&mut self.crate_id, meta.source_crate);
            let old_module = replace(&mut self.local_module, meta.source_module);
            self.introduce_generics_into_scope(meta.all_generics.clone());
            (old_crate, old_module)
        });

        self.populate_scope_from_comptime_scopes();
        let expr = self.elaborate_expression(expr).0;

        if let Some((old_crate, old_module)) = old_crate_and_module {
            self.crate_id = old_crate;
            self.local_module = old_module;
        }

        self.generics = old_generics;
        self.current_item = old_item;
        self.scopes.end_function();
        self.scopes.0.push(old_scope);
        self.check_and_pop_function_context();
        expr
    }

    fn populate_scope_from_comptime_scopes(&mut self) {
        // Take the comptime scope to be our runtime scope.
        // Iterate from global scope to the most local scope so that the
        // later definitions will naturally shadow the former.
        for scope in &self.comptime_scopes {
            for definition_id in scope.keys() {
                let definition = self.interner.definition(*definition_id);
                let name = definition.name.clone();
                let location = definition.location;

                let scope = self.scopes.get_mut_scope();
                let ident = HirIdent::non_trait_method(*definition_id, location);
                let meta = ResolverMeta { ident, num_times_used: 0, warn_if_unused: false };
                scope.add_key_value(name.clone(), meta);
            }
        }
    }
}
