use super::import::{
    allow_referencing_contracts, resolve_path_to_ns, ImportDirective, PathResolutionError,
};
use crate::Path;
use std::collections::BTreeMap;

use crate::graph::CrateId;
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleDefId, ModuleId};

pub trait PathResolver {
    /// Resolve the given path returning the resolved ModuleDefId.
    fn resolve(
        &self,
        def_maps: &BTreeMap<CrateId, CrateDefMap>,
        path: Path,
    ) -> Result<ModuleDefId, PathResolutionError>;

    fn local_module_id(&self) -> LocalModuleId;

    fn module_id(&self) -> ModuleId;
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
        def_maps: &BTreeMap<CrateId, CrateDefMap>,
        path: Path,
    ) -> Result<ModuleDefId, PathResolutionError> {
        resolve_path(def_maps, self.module_id, path)
    }

    fn local_module_id(&self) -> LocalModuleId {
        self.module_id.local_id
    }

    fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

/// Resolve the given path to a function or a type.
/// In the case of a conflict, functions are given priority
pub fn resolve_path(
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    module_id: ModuleId,
    path: Path,
) -> Result<ModuleDefId, PathResolutionError> {
    // lets package up the path into an ImportDirective and resolve it using that
    let import = ImportDirective { module_id: module_id.local_id, path, alias: None };
    let allow_referencing_contracts =
        allow_referencing_contracts(def_maps, module_id.krate, module_id.local_id);

    let def_map = &def_maps[&module_id.krate];
    let ns = resolve_path_to_ns(&import, def_map, def_maps, allow_referencing_contracts)?;

    let function = ns.values.map(|(id, _)| id);
    let id = function.or_else(|| ns.types.map(|(id, _)| id));
    Ok(id.expect("Found empty namespace"))
}
