use std::collections::{HashMap, HashSet};

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

    /// The namespace a definition occupies. Imports are tracked separately (by name only) since
    /// a `use` is one syntactic unit regardless of which namespaces the name resolves into.
    fn namespace(&self) -> Namespace {
        match self {
            UnusedItem::Function(_) | UnusedItem::Global(_) => Namespace::Value,
            UnusedItem::Struct(_)
            | UnusedItem::Enum(_)
            | UnusedItem::Trait(_)
            | UnusedItem::TypeAlias(_) => Namespace::Type,
            UnusedItem::Import => {
                unreachable!("imports are tracked by name only, not by namespace")
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct UsageTracker {
    /// Unused definitions per module, keyed by `(namespace, name)`. Noir's type and value
    /// namespaces are separate, so a type-namespace item and a value-namespace item can legally
    /// share a name within a module (e.g. `struct N` and `fn N`) without a duplicate-definition
    /// error. Keying by namespace as well as name keeps the two in separate slots, so marking one
    /// as used (e.g. constructing the struct) leaves the genuinely unused sibling tracked.
    unused_items: HashMap<ModuleId, HashMap<(Namespace, Ident), UnusedItem>>,
    /// Unused import names per module. A `use` is a single syntactic unit: the name it brings
    /// into scope may occupy both namespaces (e.g. a re-exported `struct N` and `fn N`), yet
    /// referencing it in either namespace uses the one import. Tracking imports by name alone
    /// keeps them independent of the namespace-keyed definitions above.
    unused_imports: HashMap<ModuleId, HashSet<Ident>>,
    unused_impl_functions: HashMap<FuncId, Ident>,
}

impl UsageTracker {
    /// Register a definition as unused, waiting to be marked as used later.
    /// Things that should not emit warnings should not be added at all.
    pub(crate) fn add_unused_item(
        &mut self,
        module_id: ModuleId,
        name: Ident,
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
                .entry((item.namespace(), name))
                .or_insert(item);
        }
    }

    /// Register an unused import. Imports are keyed by name only; if the imported name occupies
    /// both namespaces this is still a single import, so repeated registrations coalesce.
    pub(crate) fn add_unused_import(
        &mut self,
        module_id: ModuleId,
        name: Ident,
        visibility: ItemVisibility,
    ) {
        if visibility != ItemVisibility::Public && !name.span().is_empty() {
            self.unused_imports.entry(module_id).or_default().insert(name);
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
        // Referencing a name uses the import that brought it in, regardless of namespace.
        if let Some(imports) = self.unused_imports.get_mut(&current_mod_id) {
            imports.remove(name);
        }

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
        // Using a name in either namespace uses the import that brought it in.
        if let Some(imports) = self.unused_imports.get_mut(&current_mod_id) {
            imports.remove(name);
        }

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

    /// Get all the unused definitions per module, keyed by `(namespace, name)`.
    pub fn unused_items(&self) -> &HashMap<ModuleId, HashMap<(Namespace, Ident), UnusedItem>> {
        &self.unused_items
    }

    /// Get all the unused import names per module.
    pub fn unused_imports(&self) -> &HashMap<ModuleId, HashSet<Ident>> {
        &self.unused_imports
    }
}
