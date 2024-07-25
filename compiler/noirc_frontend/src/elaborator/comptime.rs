use crate::{
    hir_def::expr::HirIdent,
    node_interner::{DependencyId, FuncId},
};

use super::{Elaborator, FunctionContext, ResolverMeta};

impl<'context> Elaborator<'context> {
    /// Elaborate an expression from the middle of a comptime scope.
    /// When this happens we require additional information to know
    /// what variables should be in scope.
    pub fn elaborate_item_from_comptime<'a, T>(
        &'a mut self,
        current_function: Option<FuncId>,
        f: impl FnOnce(&mut Elaborator<'a>) -> T,
    ) -> T {
        // Create a fresh elaborator to ensure no state is changed from
        // this elaborator
        let mut elaborator = Elaborator::new(
            self.interner,
            self.def_maps,
            self.crate_id,
            self.debug_comptime_in_file,
        );

        elaborator.function_context.push(FunctionContext::default());
        elaborator.scopes.start_function();

        if let Some(function) = current_function {
            let meta = elaborator.interner.function_meta(&function);
            elaborator.current_item = Some(DependencyId::Function(function));
            elaborator.crate_id = meta.source_crate;
            elaborator.local_module = meta.source_module;
            elaborator.file = meta.source_file;
            elaborator.introduce_generics_into_scope(meta.all_generics.clone());
        }

        elaborator.comptime_scopes = std::mem::take(&mut self.comptime_scopes);
        elaborator.populate_scope_from_comptime_scopes();

        let result = f(&mut elaborator);
        elaborator.check_and_pop_function_context();

        self.comptime_scopes = elaborator.comptime_scopes;
        self.errors.append(&mut elaborator.errors);
        result
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
