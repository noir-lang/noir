use std::collections::BTreeMap;

use noirc_frontend::{
    ast::{Ident, ItemVisibility},
    graph::{CrateId, Dependency},
    hir::def_map::{CrateDefMap, ModuleDefId, ModuleId},
    modules::{get_parent_module, module_def_id_is_visible},
    node_interner::{NodeInterner, Reexport},
};

/// Finds a visible reexport for any ancestor module of the given ModuleDefId,
pub(crate) fn get_ancestor_module_reexport(
    module_def_id: ModuleDefId,
    visibility: ItemVisibility,
    current_module_id: ModuleId,
    interner: &NodeInterner,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    dependencies: &[Dependency],
) -> Option<Reexport> {
    let parent_module = get_parent_module(module_def_id, interner, def_maps)?;
    let reexport =
        interner.get_reexports(ModuleDefId::ModuleId(parent_module)).iter().find(|reexport| {
            module_def_id_is_visible(
                ModuleDefId::ModuleId(reexport.module_id),
                current_module_id,
                reexport.visibility,
                None,
                interner,
                def_maps,
                dependencies,
            )
        });
    if let Some(reexport) = reexport {
        return Some(reexport.clone());
    }

    // Try searching in the parent's parent module.
    let mut grandparent_module_reexport = get_ancestor_module_reexport(
        ModuleDefId::ModuleId(parent_module),
        visibility,
        current_module_id,
        interner,
        def_maps,
        dependencies,
    )?;

    // If we can find one, we need to check if ModuleDefId is actually visible from the grandparent module
    if !module_def_id_is_visible(
        module_def_id,
        current_module_id,
        visibility,
        Some(grandparent_module_reexport.module_id),
        interner,
        def_maps,
        dependencies,
    ) {
        return None;
    }

    // If we can find one we need to adjust the exported name a bit.
    let parent_module_name = &interner.try_module_attributes(&parent_module)?.name;
    grandparent_module_reexport.name = Ident::new(
        format!("{}::{}", grandparent_module_reexport.name, parent_module_name),
        grandparent_module_reexport.name.location(),
    );

    Some(grandparent_module_reexport)
}
