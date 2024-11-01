use super::import::{resolve_import, ImportDirective, PathResolution, PathResolutionResult};
use crate::ast::{ItemVisibility, Path};
use crate::node_interner::{NodeInterner, ReferenceId};
use crate::usage_tracker::UsageTracker;

use std::collections::BTreeMap;

use crate::graph::CrateId;
use crate::hir::def_map::{CrateDefMap, ModuleId};

/// Resolve the given path to a function or a type.
/// In the case of a conflict, functions are given priority
pub fn resolve_path(
    interner: &NodeInterner,
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
    let resolved_import = resolve_import(
        module_id.krate,
        &import,
        interner,
        def_maps,
        usage_tracker,
        path_references,
    )?;

    Ok(PathResolution { item: resolved_import.item, errors: resolved_import.errors })
}
