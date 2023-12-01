use std::collections::BTreeMap;

use fm::FileId;
use iter_extended::vecmap;

use crate::{
    graph::CrateId,
    hir::{
        def_collector::dc_crate::{CompilationError, UnresolvedStruct},
        def_map::ModuleId,
        Context,
    },
    node_interner::StructId,
    Generics, Ident, Type,
};

use super::{errors::ResolverError, path_resolver::StandardPathResolver, resolver::Resolver};

/// Create the mappings from TypeId -> StructType
/// so that expressions can access the fields of structs
pub(crate) fn resolve_structs(
    context: &mut Context,
    structs: BTreeMap<StructId, UnresolvedStruct>,
    crate_id: CrateId,
) -> Vec<(CompilationError, FileId)> {
    let mut errors: Vec<(CompilationError, FileId)> = vec![];
    // Resolve each field in each struct.
    // Each struct should already be present in the NodeInterner after def collection.
    for (type_id, typ) in structs {
        let file_id = typ.file_id;
        let (generics, fields, resolver_errors) = resolve_struct_fields(context, crate_id, typ);
        errors.extend(vecmap(resolver_errors, |err| (err.into(), file_id)));
        context.def_interner.update_struct(type_id, |struct_def| {
            struct_def.set_fields(fields);
            struct_def.generics = generics;
        });
    }
    errors
}

fn resolve_struct_fields(
    context: &mut Context,
    krate: CrateId,
    unresolved: UnresolvedStruct,
) -> (Generics, Vec<(Ident, Type)>, Vec<ResolverError>) {
    let path_resolver =
        StandardPathResolver::new(ModuleId { local_id: unresolved.module_id, krate });
    let file_id = unresolved.file_id;
    let (generics, fields, errors) =
        Resolver::new(&mut context.def_interner, &path_resolver, &context.def_maps, file_id)
            .resolve_struct_fields(unresolved.struct_def);
    (generics, fields, errors)
}
