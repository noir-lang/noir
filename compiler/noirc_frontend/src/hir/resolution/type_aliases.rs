use super::{path_resolver::StandardPathResolver, resolver::Resolver};
use crate::{
    graph::CrateId,
    hir::{
        def_collector::dc_crate::{CompilationError, UnresolvedTypeAlias},
        def_map::ModuleId,
        Context,
    },
    node_interner::TypeAliasId,
};
use fm::FileId;
use std::collections::BTreeMap;

pub(crate) fn resolve_type_aliases(
    context: &mut Context,
    type_aliases: BTreeMap<TypeAliasId, UnresolvedTypeAlias>,
    crate_id: CrateId,
) -> Vec<(CompilationError, FileId)> {
    let mut errors: Vec<(CompilationError, FileId)> = vec![];
    for (alias_id, unresolved_typ) in type_aliases {
        let path_resolver = StandardPathResolver::new(ModuleId {
            local_id: unresolved_typ.module_id,
            krate: crate_id,
        });
        let file = unresolved_typ.file_id;
        let (typ, generics, resolver_errors) =
            Resolver::new(&mut context.def_interner, &path_resolver, &context.def_maps, file)
                .resolve_type_alias(unresolved_typ.type_alias_def, alias_id);
        errors.extend(resolver_errors.iter().cloned().map(|e| (e.into(), file)));
        context.def_interner.set_type_alias(alias_id, typ, generics);
    }
    errors
}
