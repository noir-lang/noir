use std::collections::{HashMap, HashSet};

use noirc_errors::Location;

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
    /// Unused imports per module, keyed by the imported name *and its location*, recording which
    /// namespace(s) that one `use` brought the name into. `Ident`'s `Eq`/`Hash` ignore the location,
    /// so keying by name alone would merge two distinct `use`s of the same name (`use foo::N` and
    /// `use bar::N`) into one entry; pairing the name with its location keeps each `use` separate,
    /// both so the warning points at the right line and so using one doesn't silence the other. A
    /// `use` is a single syntactic unit, so using the name in any namespace it brought in uses that
    /// whole import (the entry is dropped, warning at most once); but using a same-named item in a
    /// namespace the import does not occupy leaves it untouched — e.g. constructing a local
    /// `struct N` must not silence an unused `use foo::N` that only imported a value `fn N`.
    unused_imports: HashMap<ModuleId, HashMap<(Ident, Location), HashSet<Namespace>>>,
    /// Unused methods from inherent impls (`impl Foo { fn bar() {} }`), keyed by `FuncId`. Unlike
    /// free functions, these don't occupy a module's `(namespace, name)` slot — they live in a
    /// type's method set — so they're tracked by id and removed when the method is called or
    /// otherwise referenced. The `Ident` is the method name, used to locate the unused warning.
    unused_impl_functions: HashMap<FuncId, Ident>,
    /// A stack of [`Journal::Suspended`] frames. Each marks a region where removals from
    /// `unused_items`/`unused_imports` are committed unconditionally, used to run a committed side
    /// effect — e.g. resolving a function's [`FuncMeta`] — without its marks being rolled back.
    ///
    /// [`FuncMeta`]: crate::hir_def::function::FuncMeta
    journal: Vec<Journal>,
}

/// A resolution-mode frame on the [`UsageTracker`]'s journal stack.
#[derive(Debug)]
enum Journal {
    /// Removals are committed unconditionally.
    Suspended,
}

/// Proof that a suspended frame is open, returned by [`UsageTracker::suspend_speculative`] and
/// consumed by [`UsageTracker::resume_speculative`]. It can only be constructed here, and
/// `#[must_use]` nudges callers to pair the suspend with a resume.
#[must_use]
pub(crate) struct SuspendTx {
    _private: (),
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

    /// Register an unused import. Imports are keyed by name and location; the imported name may
    /// occupy both namespaces (e.g. a re-exported `struct N` and `fn N`), in which case this is
    /// called once per namespace with the same name and location, accumulating into the one
    /// `use`'s entry.
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
                .entry((name.clone(), name.location()))
                .or_default()
                .insert(namespace);
        }
    }

    /// Marks the import that brought `name` into `namespace` as used. A `use` is a single
    /// syntactic unit, so using the name in any namespace it occupies uses that whole import (its
    /// entry is dropped). Each entry is keyed by the importing `use`'s location, so a same-named
    /// item imported by a *different* `use` (a different location) into another namespace is left
    /// alone.
    fn mark_import_as_used(
        &mut self,
        current_mod_id: ModuleId,
        name: &Ident,
        namespace: Namespace,
    ) {
        let Some(imports) = self.unused_imports.get_mut(&current_mod_id) else {
            return;
        };
        imports.retain(|(import_name, _location), namespaces| {
            !(import_name == name && namespaces.contains(&namespace))
        });
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

        let key = (namespace, name.clone());
        if let Some(UnusedItem::Struct(_)) =
            self.unused_items.get(&current_mod_id).and_then(|items| items.get(&key))
        {
            return;
        }

        if let Some(items) = self.unused_items.get_mut(&current_mod_id) {
            items.remove(&key);
        }
    }

    /// Marks an item as being used.
    pub(crate) fn mark_as_used(
        &mut self,
        current_mod_id: ModuleId,
        name: &Ident,
        namespace: Namespace,
    ) {
        self.mark_import_as_used(current_mod_id, name, namespace);

        let key = (namespace, name.clone());
        if let Some(items) = self.unused_items.get_mut(&current_mod_id) {
            items.remove(&key);
        }
    }

    /// Suspend rollback recording by pushing a [`Journal::Suspended`] frame: while it is on top,
    /// removals from the unused maps are committed unconditionally (never recorded for rollback).
    /// Used to run a committed side effect — e.g. resolving a function's [`FuncMeta`], which
    /// structurally strikes the function off the to-be-resolved list and so cannot be undone — from
    /// inside a speculative probe without its usage-marks being rolled back when the probe fails.
    /// Pair with [`resume_speculative`](Self::resume_speculative).
    ///
    /// [`FuncMeta`]: crate::hir_def::function::FuncMeta
    pub(crate) fn suspend_speculative(&mut self) -> SuspendTx {
        self.journal.push(Journal::Suspended);
        SuspendTx { _private: () }
    }

    /// Resume after a [`suspend_speculative`](Self::suspend_speculative), popping its suspended frame.
    pub(crate) fn resume_speculative(&mut self, _tx: SuspendTx) {
        match self.journal.pop() {
            Some(Journal::Suspended) => {}
            other => {
                panic!("resume_speculative expected a suspended frame on top, found {other:?}")
            }
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

    /// Get all the unused imports per module, keyed by the imported name and its location; the
    /// value is the set of namespaces that one `use` brought the name into.
    pub fn unused_imports(
        &self,
    ) -> &HashMap<ModuleId, HashMap<(Ident, Location), HashSet<Namespace>>> {
        &self.unused_imports
    }
}
