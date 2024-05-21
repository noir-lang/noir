use super::import::{resolve_import, ImportDirective, PathResolution, PathResolutionResult};
use crate::ast::Path;
use std::collections::BTreeMap;

use crate::graph::CrateId;
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleId};

pub trait PathResolver {
    /// Resolve the given path returning the resolved ModuleDefId.
    fn resolve(
        &self,
        def_maps: &BTreeMap<CrateId, CrateDefMap>,
        path: Path,
    ) -> PathResolutionResult;

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
    ) -> PathResolutionResult {
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
) -> PathResolutionResult {
    // lets package up the path into an ImportDirective and resolve it using that
    let import =
        ImportDirective { module_id: module_id.local_id, path, alias: None, is_prelude: false };
    let resolved_import = resolve_import(module_id.krate, &import, def_maps)?;

    let namespace = resolved_import.resolved_namespace;
    let id =
        namespace.values.or(namespace.types).map(|(id, _, _)| id).expect("Found empty namespace");

    Ok(PathResolution { module_def_id: id, error: resolved_import.error })
}
