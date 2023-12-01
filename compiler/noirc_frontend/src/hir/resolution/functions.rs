use std::{collections::BTreeMap, rc::Rc};

use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::Span;

use crate::{
    graph::CrateId,
    hir::{
        def_collector::dc_crate::{CompilationError, UnresolvedFunctions},
        def_map::{CrateDefMap, ModuleId},
    },
    node_interner::{FuncId, NodeInterner, TraitImplId},
    Type, TypeVariable,
};

use super::{path_resolver::StandardPathResolver, resolver::Resolver};

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_function_set(
    interner: &mut NodeInterner,
    crate_id: CrateId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    mut unresolved_functions: UnresolvedFunctions,
    self_type: Option<Type>,
    trait_impl_id: Option<TraitImplId>,
    impl_generics: Vec<(Rc<String>, TypeVariable, Span)>,
    errors: &mut Vec<(CompilationError, FileId)>,
) -> Vec<(FileId, FuncId)> {
    let file_id = unresolved_functions.file_id;

    let where_clause_errors =
        unresolved_functions.resolve_trait_bounds_trait_ids(def_maps, crate_id);
    errors.extend(where_clause_errors.iter().cloned().map(|e| (e.into(), file_id)));

    vecmap(unresolved_functions.functions, |(mod_id, func_id, func)| {
        let module_id = ModuleId { krate: crate_id, local_id: mod_id };
        let path_resolver = StandardPathResolver::new(module_id);

        let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file_id);
        // Must use set_generics here to ensure we re-use the same generics from when
        // the impl was originally collected. Otherwise the function will be using different
        // TypeVariables for the same generic, causing it to instantiate incorrectly.
        resolver.set_generics(impl_generics.clone());
        resolver.set_self_type(self_type.clone());
        resolver.set_trait_id(unresolved_functions.trait_id);
        resolver.set_trait_impl_id(trait_impl_id);

        // Without this, impl methods can accidentally be placed in contracts. See #3254
        if self_type.is_some() {
            resolver.set_in_contract(false);
        }

        let (hir_func, func_meta, errs) = resolver.resolve_function(func, func_id);
        interner.push_fn_meta(func_meta, func_id);
        interner.update_fn(func_id, hir_func);
        errors.extend(errs.iter().cloned().map(|e| (e.into(), file_id)));
        (file_id, func_id)
    })
}

pub(crate) fn resolve_free_functions(
    interner: &mut NodeInterner,
    crate_id: CrateId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    collected_functions: Vec<UnresolvedFunctions>,
    self_type: Option<Type>,
    errors: &mut Vec<(CompilationError, FileId)>,
) -> Vec<(FileId, FuncId)> {
    collected_functions
        .into_iter()
        .flat_map(|unresolved_functions| {
            resolve_function_set(
                interner,
                crate_id,
                def_maps,
                unresolved_functions,
                self_type.clone(),
                None,
                vec![], // no impl generics
                errors,
            )
        })
        .collect()
}
