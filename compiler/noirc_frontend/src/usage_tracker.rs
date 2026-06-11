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
    /// Unused imports per module, keyed by name, recording which namespace(s) the import brought
    /// the name into. A `use` is a single syntactic unit, so using the name in any namespace it
    /// occupies uses the one import (warning at most once). Recording the namespaces lets a use in
    /// one namespace leave an import that only occupies the other namespace untouched — e.g.
    /// constructing a local `struct N` must not silence an unused `use foo::N` that only imported
    /// a value-namespace `fn N`.
    unused_imports: HashMap<ModuleId, HashMap<Ident, HashSet<Namespace>>>,
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

    /// Register an unused import. Imports are keyed by name; the imported name may occupy both
    /// namespaces (e.g. a re-exported `struct N` and `fn N`), in which case this is called once
    /// per namespace and the namespaces accumulate into the one import entry.
    pub(crate) fn add_unused_import(
        &mut self,
        module_id: ModuleId,
        name: Ident,
        namespace: Namespace,
        visibility: ItemVisibility,
    ) {
        if visibility != ItemVisibility::Public && !name.span().is_empty() {
            self.unused_imports
                .entry(module_id)
                .or_default()
                .entry(name)
                .or_default()
                .insert(namespace);
        }
    }

    /// Marks the import that brought `name` into `namespace` as used. A `use` is a single
    /// syntactic unit, so using the name in any namespace it occupies uses the whole import; but
    /// using a same-named item in a namespace the import does not occupy leaves the import alone.
    fn mark_import_as_used(
        &mut self,
        current_mod_id: ModuleId,
        name: &Ident,
        namespace: Namespace,
    ) {
        if let Some(imports) = self.unused_imports.get_mut(&current_mod_id)
            && imports.get(name).is_some_and(|namespaces| namespaces.contains(&namespace))
        {
            imports.remove(name);
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
        self.mark_import_as_used(current_mod_id, name, namespace);

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
        self.mark_import_as_used(current_mod_id, name, namespace);

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

    /// Get all the unused imports per module, keyed by name, with the namespace(s) each occupies.
    pub fn unused_imports(&self) -> &HashMap<ModuleId, HashMap<Ident, HashSet<Namespace>>> {
        &self.unused_imports
    }
}
