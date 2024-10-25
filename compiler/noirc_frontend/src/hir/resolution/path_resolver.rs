use super::import::{resolve_import, ImportDirective, PathResolution, PathResolutionResult};
use crate::ast::{ItemVisibility, Path};
use crate::node_interner::ReferenceId;
use crate::usage_tracker::UsageTracker;

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
        usage_tracker: &mut UsageTracker,
        path_references: &mut Option<&mut Vec<ReferenceId>>,
    ) -> PathResolutionResult;

    fn local_module_id(&self) -> LocalModuleId;

    fn module_id(&self) -> ModuleId;
}

pub struct StandardPathResolver {
    // Module that we are resolving the path in
    module_id: ModuleId,
    // The module of the self type, if any (for example, the ModuleId of a struct)
    self_type_module_id: Option<ModuleId>,
}

impl StandardPathResolver {
    pub fn new(module_id: ModuleId, self_type_module_id: Option<ModuleId>) -> StandardPathResolver {
        Self { module_id, self_type_module_id }
    }
}

impl PathResolver for StandardPathResolver {
    fn resolve(
        &self,
        def_maps: &BTreeMap<CrateId, CrateDefMap>,
        path: Path,
        usage_tracker: &mut UsageTracker,
        path_references: &mut Option<&mut Vec<ReferenceId>>,
    ) -> PathResolutionResult {
        resolve_path(
            def_maps,
            self.module_id,
            self.self_type_module_id,
            path,
            usage_tracker,
            path_references,
        )
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
    self_type_module_id: Option<ModuleId>,
    path: Path,
    usage_tracker: &mut UsageTracker,
    path_references: &mut Option<&mut Vec<ReferenceId>>,
) -> PathResolutionResult {
    // lets package up the path into an ImportDirective and resolve it using that
    let import = ImportDirective {
        visibility: ItemVisibility::Private,
        module_id: module_id.local_id,
        self_type_module_id,
        path,
        alias: None,
        is_prelude: false,
    };
    let resolved_import =
        resolve_import(module_id.krate, &import, def_maps, usage_tracker, path_references)?;

    Ok(PathResolution { item: resolved_import.item, errors: resolved_import.errors })
}
