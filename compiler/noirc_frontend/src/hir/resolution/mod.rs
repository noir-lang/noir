//! This set of modules implements the second half of the name resolution pass.
//! After all definitions are collected by def_collector, resolver::Resolvers are
//! created to resolve all names within a definition. In this context 'resolving'
//! a name means validating that it has a valid definition, and that it was not
//! redefined multiple times in the same scope. Once this is validated, it is linked
//! to that definition via a matching DefinitionId. All references to the same definition
//! will have the same DefinitionId.
pub mod errors;
pub mod import;
pub mod path_resolver;
pub mod resolver;

mod functions;
mod globals;
mod impls;
mod structs;
mod traits;
mod type_aliases;

pub(crate) use functions::resolve_free_functions;
pub(crate) use globals::resolve_globals;
pub(crate) use impls::{collect_impls, resolve_impls};
pub(crate) use structs::resolve_structs;
pub(crate) use traits::{
    collect_trait_impls, resolve_trait_by_path, resolve_trait_impls, resolve_traits,
};
pub(crate) use type_aliases::resolve_type_aliases;

use crate::{
    graph::CrateId,
    hir::{
        def_collector::dc_crate::CompilationError,
        def_map::{CrateDefMap, ModuleData, ModuleId},
    },
    Shared, StructType, Type,
};
use fm::FileId;
use iter_extended::vecmap;
use resolver::Resolver;
use std::collections::BTreeMap;

fn take_errors(file_id: FileId, resolver: Resolver<'_>) -> Vec<(CompilationError, FileId)> {
    vecmap(resolver.take_errors(), |e| (e.into(), file_id))
}

fn get_module_mut(
    def_maps: &mut BTreeMap<CrateId, CrateDefMap>,
    module: ModuleId,
) -> &mut ModuleData {
    &mut def_maps.get_mut(&module.krate).unwrap().modules[module.local_id.0]
}

fn get_struct_type(typ: &Type) -> Option<&Shared<StructType>> {
    match typ {
        Type::Struct(definition, _) => Some(definition),
        _ => None,
    }
}
