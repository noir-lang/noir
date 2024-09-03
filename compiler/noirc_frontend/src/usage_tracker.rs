use rustc_hash::FxHashMap as HashMap;

use crate::{
    ast::{Ident, ItemVisibility},
    hir::def_map::ModuleId,
    macros_api::ModuleDefId,
};

#[derive(Debug, Default)]
pub struct UsageTracker {
    unused_items: HashMap<ModuleId, HashMap<Ident, ModuleDefId>>,
}

impl UsageTracker {
    pub(crate) fn add_unused_item(
        &mut self,
        module_id: ModuleId,
        name: Ident,
        item: ModuleDefId,
        visibility: ItemVisibility,
    ) {
        // Empty spans could come from implicitly injected imports, and we don't want to track those
        if visibility != ItemVisibility::Public && name.span().start() < name.span().end() {
            self.unused_items.entry(module_id).or_default().insert(name, item);
        }
    }

    pub(crate) fn mark_as_used(&mut self, current_mod_id: ModuleId, name: &Ident) {
        self.unused_items.entry(current_mod_id).or_default().remove(name);
    }

    pub(crate) fn unused_items(&self) -> &HashMap<ModuleId, HashMap<Ident, ModuleDefId>> {
        &self.unused_items
    }
}
