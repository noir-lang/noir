use std::collections::BTreeMap;

use noirc_frontend::{
    ast::ItemVisibility,
    graph::CrateId,
    hir::{
        def_map::{CrateDefMap, ModuleDefId, ModuleId},
        resolution::visibility::item_in_module_is_visible,
    },
    node_interner::NodeInterner,
};

use crate::modules::get_parent_module;

/// Returns true if the given ModuleDefId is visible from the current module, given its visibility.
/// This will in turn check if the ModuleDefId parent modules are visible from the current module.
/// If `defining_module` is Some, it will be considered as the parent of the item to check
/// (this is the case when the item is re-exported with `pub use` or similar).
pub(super) fn module_def_id_is_visible(
    module_def_id: ModuleDefId,
    current_module_id: ModuleId,
    mut visibility: ItemVisibility,
    mut defining_module: Option<ModuleId>,
    interner: &NodeInterner,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
) -> bool {
    // First find out which module we need to check.
    // If a module is trying to be referenced, it's that module. Otherwise it's the module that contains the item.
    let mut target_module_id = if let ModuleDefId::ModuleId(module_id) = module_def_id {
        Some(module_id)
    } else {
        std::mem::take(&mut defining_module).or_else(|| get_parent_module(interner, module_def_id))
    };

    // Then check if it's visible, and upwards
    while let Some(module_id) = target_module_id {
        if !item_in_module_is_visible(def_maps, current_module_id, module_id, visibility) {
            return false;
        }

        target_module_id = std::mem::take(&mut defining_module).or_else(|| {
            let module_data = &def_maps[&module_id.krate].modules()[module_id.local_id.0];
            let parent_local_id = module_data.parent;
            parent_local_id.map(|local_id| ModuleId { krate: module_id.krate, local_id })
        });

        // This is a bit strange, but the visibility is always that of the item inside another module,
        // so the visibility we update here is for the next loop check.
        visibility = interner
            .try_module_attributes(&module_id)
            .map_or(ItemVisibility::Public, |attributes| attributes.visibility);
    }

    true
}
