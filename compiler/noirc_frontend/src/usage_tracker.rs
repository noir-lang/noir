use std::collections::HashMap;

use crate::{
    ast::{Ident, ItemVisibility},
    hir::def_map::ModuleId,
    node_interner::{FuncId, GlobalId, TraitId, TypeAliasId, TypeId},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UnusedItem {
    Import,
    Function(FuncId),
    Struct(TypeId),
    Enum(TypeId),
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
            UnusedItem::Enum(_) => "enum",
            UnusedItem::Trait(_) => "trait",
            UnusedItem::TypeAlias(_) => "type alias",
            UnusedItem::Global(_) => "global",
        }
    }
}

#[derive(Debug, Default)]
pub struct UsageTracker {
    /// Unused items per module, keyed only by name. Noir's type and value namespaces are
    /// separate, so a type-namespace item and a value-namespace item can legally share a
    /// name within a module (e.g. `struct N` and `fn N`) without a duplicate-definition
    /// error. Keying by name alone conflates the two: only one is tracked, and marking
    /// either as used clears the shared slot. The worst case is a *missing* unused warning
    /// for the untracked sibling — never a missing error, since genuine same-namespace
    /// clashes are caught as duplicate definitions before reaching here.
    unused_items: HashMap<ModuleId, HashMap<Ident, UnusedItem>>,
    unused_impl_functions: HashMap<FuncId, Ident>,
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
            // Two items can share a name within a module (e.g. a global and a function),
            // which is a duplicate-definition error reported elsewhere. `HashMap::insert`
            // keeps the existing key but swaps the value, leaving the recorded location and
            // item kind describing different definitions. Keep the first item we saw so the
            // unused warning stays self-consistent.
            self.unused_items.entry(module_id).or_default().entry(name).or_insert(item);
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
        }
    }

    /// Register an inherent impl function as unused.
    pub(crate) fn add_unused_impl_function(
        &mut self,
        func_id: FuncId,
        name: Ident,
        visibility: ItemVisibility,
    ) {
        if visibility != ItemVisibility::Public && name.span().start() < name.span().end() {
            self.unused_impl_functions.insert(func_id, name);
        }
    }

    /// Marks an inherent impl function as used.
    pub(crate) fn mark_impl_function_as_used(&mut self, func_id: &FuncId) {
        self.unused_impl_functions.remove(func_id);
    }

    /// Get all the unused impl functions.
    pub fn unused_impl_functions(&self) -> &HashMap<FuncId, Ident> {
        &self.unused_impl_functions
    }

    /// Get all the unused items per module.
    pub fn unused_items(&self) -> &HashMap<ModuleId, HashMap<Ident, UnusedItem>> {
        &self.unused_items
    }
}
