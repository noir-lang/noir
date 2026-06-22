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
    /// The undo log of the speculative transaction in progress, if any. `None` means no
    /// transaction is open; `Some` records every entry removed from `unused_items`/`unused_imports`
    /// while it is open, so they can be restored on rollback. This lets a speculative
    /// path-resolution probe (which marks segments as used/referenced while resolving) undo those
    /// marks when it turns out the path wasn't what it was probing for. Only one transaction may be
    /// open at a time — see [`begin_speculative`](Self::begin_speculative).
    speculative_undo: Option<Vec<SpeculativeUndo>>,
}

/// A single removal recorded during a speculative transaction, kept so it can be re-inserted on
/// rollback.
#[derive(Debug)]
pub(crate) enum SpeculativeUndo {
    Item(ModuleId, (Namespace, Ident), UnusedItem),
    Import(ModuleId, (Ident, Location), HashSet<Namespace>),
}

/// Proof that a speculative transaction is open, returned by [`UsageTracker::begin_speculative`]
/// and consumed by [`UsageTracker::commit_speculative`] / [`UsageTracker::rollback_speculative`].
/// It can only be constructed here, so a transaction cannot be started without going through
/// `begin_speculative`, and `#[must_use]` nudges callers to finish it. Prefer driving this through
/// [`Elaborator::speculatively`](crate::elaborator::Elaborator) rather than by hand.
#[must_use]
pub(crate) struct SpeculativeTx {
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
        let recording = self.speculative_undo.is_some();
        let Some(imports) = self.unused_imports.get_mut(&current_mod_id) else {
            return;
        };
        let mut removed = Vec::new();
        imports.retain(|(import_name, location), namespaces| {
            let remove = import_name == name && namespaces.contains(&namespace);
            if remove && recording {
                removed.push(((import_name.clone(), *location), namespaces.clone()));
            }
            !remove
        });
        if let Some(undo) = &mut self.speculative_undo {
            for (key, namespaces) in removed {
                undo.push(SpeculativeUndo::Import(current_mod_id, key, namespaces));
            }
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

        let key = (namespace, name.clone());
        if let Some(UnusedItem::Struct(_)) =
            self.unused_items.get(&current_mod_id).and_then(|items| items.get(&key))
        {
            return;
        }

        let removed =
            self.unused_items.get_mut(&current_mod_id).and_then(|items| items.remove_entry(&key));
        if let Some((stored_key, item)) = removed {
            self.record_item_removal(current_mod_id, stored_key, item);
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
        let removed =
            self.unused_items.get_mut(&current_mod_id).and_then(|items| items.remove_entry(&key));
        if let Some((stored_key, item)) = removed {
            self.record_item_removal(current_mod_id, stored_key, item);
        }
    }

    /// While a speculative transaction is open, record an `unused_items` removal so it can be
    /// restored on rollback. A no-op when no transaction is in progress.
    ///
    /// `key` must be the *stored* key returned by `remove_entry`, not the lookup key built from the
    /// caller's ident. `Ident`'s `Eq`/`Hash` ignore location, so a lookup ident carries the
    /// reference's location while the stored ident carries the definition's. Recording the stored
    /// key keeps the restored entry pointing at the definition, so a later unused warning lands on
    /// the definition rather than the (rolled-back) reference site.
    fn record_item_removal(
        &mut self,
        module_id: ModuleId,
        key: (Namespace, Ident),
        item: UnusedItem,
    ) {
        if let Some(undo) = &mut self.speculative_undo {
            undo.push(SpeculativeUndo::Item(module_id, key, item));
        }
    }

    /// Begin a speculative transaction: until it is committed or rolled back, every removal from
    /// `unused_items`/`unused_imports` is recorded so it can be undone. Panics if one is already
    /// open — transactions don't nest. Prefer [`Elaborator::speculatively`] over calling this
    /// directly.
    ///
    /// [`Elaborator::speculatively`]: crate::elaborator::Elaborator
    pub(crate) fn begin_speculative(&mut self) -> SpeculativeTx {
        assert!(
            self.speculative_undo.is_none(),
            "a speculative transaction is already in progress; they do not nest"
        );
        self.speculative_undo = Some(Vec::new());
        SpeculativeTx { _private: () }
    }

    /// Commit a speculative transaction, keeping every change made while it was open.
    pub(crate) fn commit_speculative(&mut self, _tx: SpeculativeTx) {
        self.speculative_undo = None;
    }

    /// Temporarily detach any in-progress speculative undo log, returning it so it can be restored
    /// with [`resume_speculative`](Self::resume_speculative). While detached, removals are committed
    /// unconditionally (never recorded for rollback). Used to run a committed side effect — e.g.
    /// resolving a function's [`FuncMeta`], which structurally strikes the function off the
    /// to-be-resolved list and so cannot be undone — from inside a speculative probe without its
    /// usage-marks being rolled back when the probe fails.
    ///
    /// [`FuncMeta`]: crate::hir_def::function::FuncMeta
    pub(crate) fn suspend_speculative(&mut self) -> Option<Vec<SpeculativeUndo>> {
        self.speculative_undo.take()
    }

    /// Restore an undo log detached by [`suspend_speculative`](Self::suspend_speculative).
    pub(crate) fn resume_speculative(&mut self, undo: Option<Vec<SpeculativeUndo>>) {
        self.speculative_undo = undo;
    }

    /// Roll back a speculative transaction, restoring every entry removed while it was open.
    pub(crate) fn rollback_speculative(&mut self, _tx: SpeculativeTx) {
        let undo =
            self.speculative_undo.take().expect("speculative transaction must be in progress");
        for entry in undo.into_iter().rev() {
            match entry {
                SpeculativeUndo::Item(module_id, key, item) => {
                    self.unused_items.entry(module_id).or_default().insert(key, item);
                }
                SpeculativeUndo::Import(module_id, key, namespaces) => {
                    self.unused_imports.entry(module_id).or_default().insert(key, namespaces);
                }
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
