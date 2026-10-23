use crate::{
    ast::{Ident, ItemVisibility},
    graph::Dependency,
    hir::{
        def_map::{DefMaps, ModuleDefId, ModuleId},
        resolution::visibility::item_in_module_is_visible,
    },
    node_interner::{NodeInterner, Reexport, ReferenceId},
};

/// Returns the `ModuleId` a `ModuleDefId` is in.
pub fn get_parent_module(
    module_def_id: ModuleDefId,
    interner: &NodeInterner,
    def_maps: &DefMaps,
) -> Option<ModuleId> {
    match module_def_id {
        ModuleDefId::ModuleId(id) => id.parent(def_maps),
        ModuleDefId::FunctionId(id) => {
            let func_meta = interner.function_meta(&id);
            Some(ModuleId { krate: func_meta.source_crate, local_id: func_meta.source_module })
        }
        ModuleDefId::TypeId(id) => interner.get_type(id).borrow().id.module_id().parent(def_maps),
        ModuleDefId::TypeAliasId(id) => Some(interner.get_type_alias(id).borrow().module_id),
        ModuleDefId::TraitId(id) => interner.get_trait(id).id.0.parent(def_maps),
        ModuleDefId::TraitAssociatedTypeId(id) => {
            let trait_id = interner.get_trait_associated_type(id).trait_id;
            interner.get_trait(trait_id).id.0.parent(def_maps)
        }
        ModuleDefId::GlobalId(id) => {
            let global = interner.get_global(id);
            Some(ModuleId { krate: global.crate_id, local_id: global.local_id })
        }
    }
}

pub fn module_def_id_to_reference_id(module_def_id: ModuleDefId) -> ReferenceId {
    match module_def_id {
        ModuleDefId::ModuleId(id) => ReferenceId::Module(id),
        ModuleDefId::FunctionId(id) => ReferenceId::Function(id),
        ModuleDefId::TypeId(id) => ReferenceId::Type(id),
        ModuleDefId::TypeAliasId(id) => ReferenceId::Alias(id),
        ModuleDefId::TraitId(id) => ReferenceId::Trait(id),
        ModuleDefId::TraitAssociatedTypeId(id) => ReferenceId::TraitAssociatedType(id),
        ModuleDefId::GlobalId(id) => ReferenceId::Global(id),
    }
}

/// Returns true if the given `ModuleDefId` is visible from the current module, given its visibility.
///
/// This will in turn check if the `ModuleDefId` parent modules are visible from the current module.
/// If `defining_module` is Some, it will be considered as the parent of the item to check
/// (this is the case when the item is re-exported with `pub use` or similar).
pub fn module_def_id_is_visible(
    module_def_id: ModuleDefId,
    current_module_id: ModuleId,
    visibility: ItemVisibility,
    defining_module: Option<ModuleId>,
    interner: &NodeInterner,
    def_maps: &DefMaps,
    dependencies: &[Dependency],
) -> bool {
    module_def_id_is_visible_impl(
        module_def_id,
        current_module_id,
        visibility,
        defining_module,
        interner,
        def_maps,
        Some(dependencies),
    )
}

/// Like `module_def_id_is_visible`, but only considers visibility modifiers and ignores whether
/// the defining crate is reachable through the current crate's dependency edges.
///
/// This is used by the macro-spliced-type leak check: a maximally-public type that lives in a
/// transitive dependency is legitimately part of the public API surface even though it cannot be
/// named by an explicit path from the current crate (which would require a direct dependency edge).
/// The leak check only cares whether the type's own visibility (and its enclosing modules') is
/// public enough, not about path reachability.
pub fn module_def_id_is_visible_ignoring_dependencies(
    module_def_id: ModuleDefId,
    current_module_id: ModuleId,
    visibility: ItemVisibility,
    defining_module: Option<ModuleId>,
    interner: &NodeInterner,
    def_maps: &DefMaps,
) -> bool {
    module_def_id_is_visible_impl(
        module_def_id,
        current_module_id,
        visibility,
        defining_module,
        interner,
        def_maps,
        None,
    )
}

/// When `dependencies` is `Some`, an item in a crate that is neither the current crate nor one of
/// its dependencies is considered not visible. When `None`, only the visibility modifiers along the
/// module chain are checked.
fn module_def_id_is_visible_impl(
    module_def_id: ModuleDefId,
    current_module_id: ModuleId,
    mut visibility: ItemVisibility,
    mut defining_module: Option<ModuleId>,
    interner: &NodeInterner,
    def_maps: &DefMaps,
    dependencies: Option<&[Dependency]>,
) -> bool {
    // First find out which module we need to check.
    // If a module is trying to be referenced, it's that module. Otherwise it's the module that contains the item.
    let mut target_module_id = if let ModuleDefId::ModuleId(module_id) = module_def_id {
        Some(module_id)
    } else {
        std::mem::take(&mut defining_module)
            .or_else(|| get_parent_module(module_def_id, interner, def_maps))
    };

    // Then check if it's visible, and upwards
    while let Some(module_id) = target_module_id {
        if !item_in_module_is_visible(def_maps, current_module_id, module_id, visibility) {
            return false;
        }

        // If the target module isn't in the same crate as `module_id` or isn't in one of its
        // dependencies, then it's not visible.
        if let Some(dependencies) = dependencies
            && module_id.krate != current_module_id.krate
            && dependencies.iter().all(|dep| dep.crate_id != module_id.krate)
        {
            return false;
        }

        target_module_id = std::mem::take(&mut defining_module).or_else(|| {
            let module_data = &def_maps[&module_id.krate][module_id.local_id];
            let parent_local_id = module_data.parent;
            parent_local_id.map(|local_id| ModuleId { krate: module_id.krate, local_id })
        });

        // This is a bit strange, but the visibility is always that of the item inside another module,
        // so the visibility we update here is for the next loop check.
        visibility = interner
            .try_module_attributes(module_id)
            .map_or(ItemVisibility::Public, |attributes| attributes.visibility);
    }

    true
}

/// Finds a visible reexport for any ancestor module of the given `ModuleDefId`,
pub fn get_ancestor_module_reexport(
    module_def_id: ModuleDefId,
    visibility: ItemVisibility,
    current_module_id: ModuleId,
    interner: &NodeInterner,
    def_maps: &DefMaps,
    dependencies: &[Dependency],
) -> Option<Reexport> {
    get_ancestor_module_reexport_impl(
        module_def_id,
        visibility,
        current_module_id,
        interner,
        def_maps,
        Some(dependencies),
    )
}

/// Like `get_ancestor_module_reexport`, but ignores whether the defining crate is reachable through
/// the current crate's dependency edges (see `module_def_id_is_visible_ignoring_dependencies`).
pub fn get_ancestor_module_reexport_ignoring_dependencies(
    module_def_id: ModuleDefId,
    visibility: ItemVisibility,
    current_module_id: ModuleId,
    interner: &NodeInterner,
    def_maps: &DefMaps,
) -> Option<Reexport> {
    get_ancestor_module_reexport_impl(
        module_def_id,
        visibility,
        current_module_id,
        interner,
        def_maps,
        None,
    )
}

fn get_ancestor_module_reexport_impl(
    module_def_id: ModuleDefId,
    visibility: ItemVisibility,
    current_module_id: ModuleId,
    interner: &NodeInterner,
    def_maps: &DefMaps,
    dependencies: Option<&[Dependency]>,
) -> Option<Reexport> {
    let parent_module = get_parent_module(module_def_id, interner, def_maps)?;
    let reexport =
        interner.get_reexports(ModuleDefId::ModuleId(parent_module)).iter().find(|reexport| {
            module_def_id_is_visible_impl(
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
    let mut grandparent_module_reexport = get_ancestor_module_reexport_impl(
        ModuleDefId::ModuleId(parent_module),
        visibility,
        current_module_id,
        interner,
        def_maps,
        dependencies,
    )?;

    // If we can find one, we need to check if ModuleDefId is actually visible from the grandparent module
    if !module_def_id_is_visible_impl(
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
    let parent_module_name = &interner.try_module_attributes(parent_module)?.name;
    grandparent_module_reexport.name = Ident::new(
        format!("{}::{}", grandparent_module_reexport.name, parent_module_name),
        grandparent_module_reexport.name.location(),
    );

    Some(grandparent_module_reexport)
}
