use std::collections::HashMap;

use crate::{
    ast::{Ident, ItemVisibility},
    hir::def_map::ModuleId,
    node_interner::{FuncId, GlobalId, StructId, TraitId, TypeAliasId},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UnusedItem {
    Import,
    Function(FuncId),
    Struct(StructId),
    Trait(TraitId),
    TypeAlias(TypeAliasId),
    Global(GlobalId),
}

impl UnusedItem {
    pub fn item_type(&self) -> &'static str {
        match self {
            UnusedItem::Import => "import",
            UnusedItem::Function(_) => "function",
            UnusedItem::Struct(_) => "struct",
            UnusedItem::Trait(_) => "trait",
            UnusedItem::TypeAlias(_) => "type alias",
            UnusedItem::Global(_) => "global",
        }
    }
}

#[derive(Debug, Default)]
pub struct UsageTracker {
    unused_items: HashMap<ModuleId, HashMap<Ident, UnusedItem>>,
}

impl UsageTracker {
    /// Register an item as unused, waiting to be marked as used later.
    /// Things that should not emit warnings should not be added at all.
    pub(crate) fn add_unused_item(
        &mut self,
        module_id: ModuleId,
        name: Ident,
        item: UnusedItem,
        visibility: ItemVisibility,
    ) {
        // Empty spans could come from implicitly injected imports, and we don't want to track those
        if visibility != ItemVisibility::Public && name.span().start() < name.span().end() {
            self.unused_items.entry(module_id).or_default().insert(name, item);
        }
    }

    /// Marks an item as being referenced. This doesn't always makes the item as used. For example
    /// if a struct is referenced it will still be considered unused unless it's constructed somewhere.
    pub(crate) fn mark_as_referenced(&mut self, current_mod_id: ModuleId, name: &Ident) {
        let Some(items) = self.unused_items.get_mut(&current_mod_id) else {
            return;
        };

        let Some(unused_item) = items.get(name) else {
            return;
        };

        if let UnusedItem::Struct(_) = unused_item {
            return;
        }

        items.remove(name);
    }

    /// Marks an item as being used.
    pub(crate) fn mark_as_used(&mut self, current_mod_id: ModuleId, name: &Ident) {
        if let Some(items) = self.unused_items.get_mut(&current_mod_id) {
            items.remove(name);
        };
    }

    /// Get all the unused items per module.
    pub fn unused_items(&self) -> &HashMap<ModuleId, HashMap<Ident, UnusedItem>> {
        &self.unused_items
    }
}
