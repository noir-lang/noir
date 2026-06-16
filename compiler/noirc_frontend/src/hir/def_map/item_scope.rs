use super::{ModuleDefId, ModuleId, Namespace, namespace::PerNs};
use crate::ast::{Ident, ItemVisibility};
use crate::node_interner::FuncId;

use std::collections::{BTreeMap, btree_map};

/// A single [Ident]'s definition in a namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NamespaceItem {
    pub id: ModuleDefId,
    pub visibility: ItemVisibility,
    /// Whether this definition was brought into scope by the stdlib prelude.
    pub is_prelude: bool,
}

/// All the definitions of [Ident]s in scope, either as `types` or `values`.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct ItemScope {
    types: BTreeMap<Ident, NamespaceItem>,
    values: BTreeMap<Ident, NamespaceItem>,
    defs: Vec<ModuleDefId>,
}

impl ItemScope {
    /// Add an [Ident] and its [`ModuleDefId`] to the namespace,
    /// and also push the definition to the `defs`.
    pub fn add_definition(
        &mut self,
        name: Ident,
        visibility: ItemVisibility,
        mod_def: ModuleDefId,
    ) -> Result<(), (Ident, Ident)> {
        self.add_item_to_namespace(name, visibility, mod_def, false)?;
        self.defs.push(mod_def);
        Ok(())
    }

    /// Add an [Ident] and its [`ModuleDefId`] to either `types` or `values`,
    /// depending on what its definition is.
    ///
    /// Returns an `Err` with `(old_item, new_item)` if there is already an
    /// item in the namespace with that exact name.
    pub fn add_item_to_namespace(
        &mut self,
        name: Ident,
        visibility: ItemVisibility,
        mod_def: ModuleDefId,
        is_prelude: bool,
    ) -> Result<(), (Ident, Ident)> {
        let add_item = |map: &mut BTreeMap<Ident, NamespaceItem>| {
            if let btree_map::Entry::Occupied(mut o) = map.entry(name.clone()) {
                // Generally we want to reject having two of the same ident in the same namespace.
                // The exception to this is when we're explicitly importing something
                // which exists in the Noir stdlib prelude.
                //
                // In this case we ignore the prelude and favour the explicit import.
                if o.get().is_prelude && !is_prelude {
                    // Explicit import or definition overrides prelude
                    *o.get_mut() = NamespaceItem { id: mod_def, visibility, is_prelude };
                    Ok(())
                } else if is_prelude {
                    // Prelude cannot override anything: silently drop prelude import
                    Ok(())
                } else {
                    // Two non-prelude definitions: genuine duplicate
                    let old_ident = o.key();
                    Err((old_ident.clone(), name))
                }
            } else {
                map.insert(name, NamespaceItem { id: mod_def, visibility, is_prelude });
                Ok(())
            }
        };

        match mod_def.namespace() {
            Namespace::Type => add_item(&mut self.types),
            Namespace::Value => add_item(&mut self.values),
        }
    }

    /// Look up an [Ident] in `types`, and return it _iff_ it's a [`ModuleDefId::ModuleId`].
    pub fn find_module_with_name(&self, mod_name: &Ident) -> Option<&ModuleId> {
        match &self.types.get(mod_name)?.id {
            ModuleDefId::ModuleId(id) => Some(id),
            _ => None,
        }
    }

    /// Look up an [Ident] in `values`, then return the [`FuncId`] if the definition is a [`ModuleDefId::FunctionId`].
    pub fn find_func_with_name(&self, func_name: &Ident) -> Option<FuncId> {
        match Self::find_name_in(func_name, &self.values)?.id {
            ModuleDefId::FunctionId(id) => Some(id),
            _ => None,
        }
    }

    /// Look for an [Ident] in both `types` and `values`.
    ///
    /// Returns the preferred, unambiguous result in both.
    pub fn find_name(&self, name: &Ident) -> PerNs {
        PerNs {
            types: Self::find_name_in(name, &self.types).copied(),
            values: Self::find_name_in(name, &self.values).copied(),
        }
    }

    /// All [Ident]s in `types` and `values`.
    pub fn names(&self) -> impl Iterator<Item = &Ident> {
        self.types.keys().chain(self.values.keys())
    }

    pub fn definitions(&self) -> &[ModuleDefId] {
        &self.defs
    }

    pub fn types(&self) -> &BTreeMap<Ident, NamespaceItem> {
        &self.types
    }

    pub fn values(&self) -> &BTreeMap<Ident, NamespaceItem> {
        &self.values
    }

    /// Look up an [Ident] in `types` or `values`.
    fn find_name_in<'a>(
        name: &Ident,
        map: &'a BTreeMap<Ident, NamespaceItem>,
    ) -> Option<&'a NamespaceItem> {
        map.get(name)
    }

    /// Clears all definitions in this scope.
    /// This isn't used in the compiler. It's only used in the LSP server
    /// when a file is changed, to clear out all definitions that are meant to be
    /// replaced with new ones from the changed file.
    pub(super) fn clear(&mut self) {
        self.types.clear();
        self.values.clear();
        self.defs.clear();
    }
}
