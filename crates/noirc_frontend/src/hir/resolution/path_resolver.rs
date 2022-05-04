use super::import::{resolve_path_to_ns, ImportDirective, PathResolution};
use crate::{Ident, Path};
use std::collections::HashMap;

use crate::graph::CrateId;
use crate::hir::def_map::{CrateDefMap, ModuleDefId, ModuleId};

pub trait PathResolver {
    fn resolve(
        &self,
        def_maps: &HashMap<CrateId, CrateDefMap>,
        path: Path,
    ) -> Result<Option<ModuleDefId>, Ident>;
}

pub struct StandardPathResolver {
    // Module that we are resolving the path in
    module_id: ModuleId,
}

impl StandardPathResolver {
    pub fn new(module_id: ModuleId) -> StandardPathResolver {
        Self { module_id }
    }
}

impl PathResolver for StandardPathResolver {
    fn resolve(
        &self,
        def_maps: &HashMap<CrateId, CrateDefMap>,
        path: Path,
    ) -> Result<Option<ModuleDefId>, Ident> {
        resolve_path(def_maps, self.module_id, path)
    }
}

/// Resolve the given path to a function or a type.
/// In the case of a conflict, functions are given priority
pub fn resolve_path(
    def_maps: &HashMap<CrateId, CrateDefMap>,
    module_id: ModuleId,
    path: Path,
) -> Result<Option<ModuleDefId>, Ident> {
    // lets package up the path into an ImportDirective and resolve it using that
    let import = ImportDirective { module_id: module_id.local_id, path, alias: None };

    let def_map = &def_maps[&module_id.krate];
    let path_res = resolve_path_to_ns(&import, def_map, def_maps);
    let ns = match path_res {
        PathResolution::Unresolved(seg) => return Err(seg),
        PathResolution::Resolved(ns) => ns,
    };

    let function = ns.values.map(|(id, _)| id);
    Ok(function.or_else(|| ns.types.map(|(id, _)| id)))
}
