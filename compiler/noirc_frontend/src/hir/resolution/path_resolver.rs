use super::import::{resolve_import, ImportDirective, PathResolution, PathResolutionResult};
use crate::ast::Path;
use crate::node_interner::ReferenceId;
use std::collections::BTreeMap;

use crate::graph::CrateId;
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleId};

pub trait PathResolver {
    /// Resolve the given path returning the resolved ModuleDefId.
    /// If `path_references` is `Some`, a `ReferenceId` for each segment in `path`
    /// will be resolved and pushed (some entries will be None if they don't refer to
    /// a module or type).
    fn resolve(
        &self,
        def_maps: &BTreeMap<CrateId, CrateDefMap>,
        path: Path,
        path_references: &mut Option<&mut Vec<Option<ReferenceId>>>,
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
        path_references: &mut Option<&mut Vec<Option<ReferenceId>>>,
    ) -> PathResolutionResult {
        resolve_path(def_maps, self.module_id, path, path_references)
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
    path_references: &mut Option<&mut Vec<Option<ReferenceId>>>,
) -> PathResolutionResult {
    // lets package up the path into an ImportDirective and resolve it using that
    let import =
        ImportDirective { module_id: module_id.local_id, path, alias: None, is_prelude: false };
    let resolved_import = resolve_import(module_id.krate, &import, def_maps, path_references)?;

    let namespace = resolved_import.resolved_namespace;
    let id =
        namespace.values.or(namespace.types).map(|(id, _, _)| id).expect("Found empty namespace");

    Ok(PathResolution { module_def_id: id, error: resolved_import.error })
}
