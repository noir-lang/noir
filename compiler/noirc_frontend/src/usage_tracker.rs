use std::collections::HashMap;

use crate::{
    ast::{Ident, ItemVisibility},
    hir::def_map::{ModuleId, Namespace},
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

    /// The namespace this item occupies. An import can resolve into either namespace, so its
    /// namespace is determined by the imported definition at the call site rather than here.
    fn namespace(&self) -> Option<Namespace> {
        match self {
            UnusedItem::Function(_) | UnusedItem::Global(_) => Some(Namespace::Value),
            UnusedItem::Struct(_)
            | UnusedItem::Enum(_)
            | UnusedItem::Trait(_)
            | UnusedItem::TypeAlias(_) => Some(Namespace::Type),
            UnusedItem::Import => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct UsageTracker {
    /// Unused items per module, keyed by `(namespace, name)`. Noir's type and value namespaces
    /// are separate, so a type-namespace item and a value-namespace item can legally share a
    /// name within a module (e.g. `struct N` and `fn N`) without a duplicate-definition error.
    /// Keying by namespace as well as name keeps the two in separate slots, so marking one as
    /// used (e.g. constructing the struct) leaves the genuinely unused sibling tracked.
    unused_items: HashMap<ModuleId, HashMap<(Namespace, Ident), UnusedItem>>,
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
        let namespace = item.namespace().expect("imports must be registered via add_unused_import");
        self.insert_unused_item(module_id, name, namespace, item, visibility);
    }

    /// Register an unused import. Unlike other items, an import can resolve into either
    /// namespace, so the caller supplies the namespace of the imported definition.
    pub(crate) fn add_unused_import(
        &mut self,
        module_id: ModuleId,
        name: Ident,
        namespace: Namespace,
        visibility: ItemVisibility,
    ) {
        self.insert_unused_item(module_id, name, namespace, UnusedItem::Import, visibility);
    }

    fn insert_unused_item(
        &mut self,
        module_id: ModuleId,
        name: Ident,
        namespace: Namespace,
        item: UnusedItem,
        visibility: ItemVisibility,
    ) {
        // Empty spans could come from implicitly injected imports, and we don't want to track those
        if visibility != ItemVisibility::Public && !name.span().is_empty() {
            // A genuine same-namespace clash is a duplicate-definition error reported elsewhere;
            // `entry(..).or_insert` keeps the first item we saw so the unused warning's recorded
            // location and item kind stay self-consistent.
            self.unused_items
                .entry(module_id)
                .or_default()
                .entry((namespace, name))
                .or_insert(item);
        }
    }

    /// Marks an item as being referenced. This doesn't always makes the item as used. For example
    /// if a struct is referenced it will still be considered unused unless it's constructed somewhere.
    pub(crate) fn mark_as_referenced(
        &mut self,
        current_mod_id: ModuleId,
        name: &Ident,
        namespace: Namespace,
    ) {
        let Some(items) = self.unused_items.get_mut(&current_mod_id) else {
            return;
        };

        let key = (namespace, name.clone());
        let Some(unused_item) = items.get(&key) else {
            return;
        };

        if let UnusedItem::Struct(_) = unused_item {
            return;
        }

        items.remove(&key);
    }

    /// Marks an item as being used.
    pub(crate) fn mark_as_used(
        &mut self,
        current_mod_id: ModuleId,
        name: &Ident,
        namespace: Namespace,
    ) {
        if let Some(items) = self.unused_items.get_mut(&current_mod_id) {
            items.remove(&(namespace, name.clone()));
        }
    }

    /// Register an inherent impl function as unused.
    pub(crate) fn add_unused_impl_function(
        &mut self,
        func_id: FuncId,
        name: Ident,
        visibility: ItemVisibility,
    ) {
        if visibility != ItemVisibility::Public && !name.span().is_empty() {
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

    /// Get all the unused items per module, keyed by `(namespace, name)`.
    pub fn unused_items(&self) -> &HashMap<ModuleId, HashMap<(Namespace, Ident), UnusedItem>> {
        &self.unused_items
    }
}
