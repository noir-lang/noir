use super::import::{
    allow_referencing_contracts, check_can_reference_function, resolve_path_to_ns, ImportDirective,
    PathResolutionError,
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
    let last_path_segment = path.last_segment();
    let import =
        ImportDirective { module_id: module_id.local_id, path, alias: None, is_prelude: false };
    let allow_referencing_contracts =
        allow_referencing_contracts(def_maps, module_id.krate, module_id.local_id);

    let (resolved_module, ns) = resolve_path_to_ns(
        &import,
        module_id.krate,
        module_id.krate,
        def_maps,
        allow_referencing_contracts,
    )?;

    let (id, visibility) = ns
        .values
        .or(ns.types)
        .map(|(id, visibility, _)| (id, visibility))
        .expect("Found empty namespace");

    check_can_reference_function(
        def_maps,
        module_id.krate,
        module_id.local_id,
        resolved_module,
        visibility,
        &last_path_segment,
    )?;

    Ok(id)
}
