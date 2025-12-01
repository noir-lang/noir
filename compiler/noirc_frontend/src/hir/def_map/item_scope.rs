use super::{ModuleDefId, ModuleId, namespace::PerNs};
use crate::ast::{Ident, ItemVisibility};
use crate::node_interner::{FuncId, TraitId};

use std::collections::{BTreeMap, btree_map};
use std::collections::{HashMap, hash_map};

/// Definitions of an [Ident]: it can be a standalone without a [TraitId],
/// or it can appear across multiple traits.
type Scope = HashMap<Option<TraitId>, (ModuleDefId, ItemVisibility, bool /*is_prelude*/)>;

/// All the definitions of [Ident]s in scope, either as `types` or `values`.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct ItemScope {
    types: BTreeMap<Ident, Scope>,
    values: BTreeMap<Ident, Scope>,
    defs: Vec<ModuleDefId>,
}

impl ItemScope {
    /// Add an [Ident] and its [ModuleDefId] to the namespace,
    /// and also push the definition to the `defs`.
    pub fn add_definition(
        &mut self,
        name: Ident,
        visibility: ItemVisibility,
        mod_def: ModuleDefId,
        trait_id: Option<TraitId>,
    ) -> Result<(), (Ident, Ident)> {
        self.add_item_to_namespace(name, visibility, mod_def, trait_id, false)?;
        self.defs.push(mod_def);
        Ok(())
    }

    /// Add an [Ident] and its [ModuleDefId] to either `types` or `values`,
    /// depending on what its definition is.
    ///
    /// Returns an `Err` with `(old_item, new_item)` if there is already an
    /// item in the namespace with that exact name.
    pub fn add_item_to_namespace(
        &mut self,
        name: Ident,
        visibility: ItemVisibility,
        mod_def: ModuleDefId,
        trait_id: Option<TraitId>,
        is_prelude: bool,
    ) -> Result<(), (Ident, Ident)> {
        let add_item = |map: &mut BTreeMap<Ident, Scope>| {
            if let btree_map::Entry::Occupied(mut o) = map.entry(name.clone()) {
                let trait_hashmap = o.get_mut();
                if let hash_map::Entry::Occupied(mut n) = trait_hashmap.entry(trait_id) {
                    // Generally we want to reject having two of the same ident in the same namespace.
                    // The exception to this is when we're explicitly importing something
                    // which exists in the Noir stdlib prelude.
                    //
                    // In this case we ignore the prelude and favour the explicit import.
                    let is_prelude = std::mem::replace(&mut n.get_mut().2, is_prelude);
                    let old_ident = o.key();

                    if is_prelude { Ok(()) } else { Err((old_ident.clone(), name)) }
                } else {
                    trait_hashmap.insert(trait_id, (mod_def, visibility, is_prelude));
                    Ok(())
                }
            } else {
                let mut trait_hashmap = HashMap::new();
                trait_hashmap.insert(trait_id, (mod_def, visibility, is_prelude));
                map.insert(name, trait_hashmap);
                Ok(())
            }
        };

        match mod_def {
            ModuleDefId::ModuleId(_) => add_item(&mut self.types),
            ModuleDefId::FunctionId(_) => add_item(&mut self.values),
            ModuleDefId::TypeId(_) => add_item(&mut self.types),
            ModuleDefId::TypeAliasId(_) => add_item(&mut self.types),
            ModuleDefId::TraitId(_) => add_item(&mut self.types),
            ModuleDefId::TraitAssociatedTypeId(_) => add_item(&mut self.types),
            ModuleDefId::GlobalId(_) => add_item(&mut self.values),
        }
    }

    /// Look up an [Ident] in `types` with no [TraitId], and return it _iff_ it's a [ModuleDefId::ModuleId].
    pub fn find_module_with_name(&self, mod_name: &Ident) -> Option<&ModuleId> {
        let (module_def, _, _) = self.types.get(mod_name)?.get(&None)?;
        match module_def {
            ModuleDefId::ModuleId(id) => Some(id),
            _ => None,
        }
    }

    /// Look up an [Ident] in `values` with no [TraitId], then return the [FuncId]
    /// if the definition is a [ModuleDef::FunctionId] in the following order:
    /// * if a definition without a [TraitId] exists, use that
    /// * if there is a single trait definition, use that
    /// * otherwise return nothing, as it is ambiguous
    pub fn find_func_with_name(&self, func_name: &Ident) -> Option<FuncId> {
        let trait_hashmap = self.values.get(func_name)?;
        // methods introduced without trait take priority and hide methods with the same name that come from a trait
        let a = trait_hashmap.get(&None);
        match a {
            Some((module_def, _, _)) => match module_def {
                ModuleDefId::FunctionId(id) => Some(*id),
                _ => None,
            },
            None => {
                if trait_hashmap.len() == 1 {
                    let (module_def, _, _) = trait_hashmap.get(trait_hashmap.keys().last()?)?;
                    match module_def {
                        ModuleDefId::FunctionId(id) => Some(*id),
                        _ => None,
                    }
                } else {
                    // ambiguous name (multiple traits, containing the same function name)
                    None
                }
            }
        }
    }

    /// Look for an [Ident] in both `types` and `values`.
    ///
    /// Returns the preferred, unambiguous result in both.
    pub fn find_name(&self, name: &Ident) -> PerNs {
        PerNs {
            types: Self::find_name_in(name, &self.types).cloned(),
            values: Self::find_name_in(name, &self.values).cloned(),
        }
    }

    /// Look for an [Ident] in both `types` and `values`,
    ///
    /// It returns the entry matching the `trait_id`, that is, either the standalone definition,
    /// or one in a specific trait (regardless of the presence of other traits).
    pub fn find_name_for_trait_id(&self, name: &Ident, trait_id: &Option<TraitId>) -> PerNs {
        PerNs {
            types: self.types.get(name).and_then(|t| t.get(trait_id)).cloned(),
            values: self.values.get(name).and_then(|v| v.get(trait_id)).cloned(),
        }
    }

    /// All [Ident]s in `types` and `values`.
    pub fn names(&self) -> impl Iterator<Item = &Ident> {
        self.types.keys().chain(self.values.keys())
    }

    pub fn definitions(&self) -> &[ModuleDefId] {
        &self.defs
    }

    pub fn types(&self) -> &BTreeMap<Ident, Scope> {
        &self.types
    }

    pub fn values(&self) -> &BTreeMap<Ident, Scope> {
        &self.values
    }

    pub fn remove_definition(&mut self, name: &Ident) {
        self.types.remove(name);
        self.values.remove(name);
    }

    /// Look up an [Ident] in `types` or `values`:
    /// * if a definition without a [TraitId] exists, return that
    /// * if there is exactly 1 definition with a [TraitId], return that
    /// * otherwise return nothing, as the name is ambiguous, exists in multiple traits
    fn find_name_in<'a>(
        name: &Ident,
        map: &'a BTreeMap<Ident, Scope>,
    ) -> Option<&'a (ModuleDefId, ItemVisibility, bool)> {
        if let Some(t) = map.get(name) {
            if let Some(tt) = t.get(&None) {
                Some(tt)
            } else if t.len() == 1 {
                t.values().last()
            } else {
                None
            }
        } else {
            None
        }
    }
}
