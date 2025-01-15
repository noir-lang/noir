use super::{namespace::PerNs, ModuleDefId, ModuleId};
use crate::ast::{Ident, ItemVisibility};
use crate::node_interner::{FuncId, TraitId};

use std::collections::{hash_map::Entry, HashMap};

type Scope = HashMap<Option<TraitId>, (ModuleDefId, ItemVisibility, bool /*is_prelude*/)>;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct ItemScope {
    types: HashMap<Ident, Scope>,
    values: HashMap<Ident, Scope>,

    defs: Vec<ModuleDefId>,
}

impl ItemScope {
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

    /// Returns an Err if there is already an item
    /// in the namespace with that exact name.
    /// The Err will return (old_item, new_item)
    pub fn add_item_to_namespace(
        &mut self,
        name: Ident,
        visibility: ItemVisibility,
        mod_def: ModuleDefId,
        trait_id: Option<TraitId>,
        is_prelude: bool,
    ) -> Result<(), (Ident, Ident)> {
        let add_item = |map: &mut HashMap<Ident, Scope>| {
            if let Entry::Occupied(mut o) = map.entry(name.clone()) {
                let trait_hashmap = o.get_mut();
                if let Entry::Occupied(mut n) = trait_hashmap.entry(trait_id) {
                    // Generally we want to reject having two of the same ident in the same namespace.
                    // The exception to this is when we're explicitly importing something
                    // which exists in the Noir stdlib prelude.
                    //
                    // In this case we ignore the prelude and favour the explicit import.
                    let is_prelude = std::mem::replace(&mut n.get_mut().2, is_prelude);
                    let old_ident = o.key();

                    if is_prelude {
                        Ok(())
                    } else {
                        Err((old_ident.clone(), name))
                    }
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
            ModuleDefId::ModuleId(_)
            | ModuleDefId::TypeId(_)
            | ModuleDefId::TypeAliasId(_)
            | ModuleDefId::TraitId(_) => add_item(&mut self.types),
            ModuleDefId::FunctionId(_) | ModuleDefId::GlobalId(_) => add_item(&mut self.values),
        }
    }

    pub fn find_module_with_name(&self, mod_name: &Ident) -> Option<&ModuleId> {
        let (module_def, _, _) = self.types.get(mod_name)?.get(&None)?;
        match module_def {
            ModuleDefId::ModuleId(id) => Some(id),
            _ => None,
        }
    }

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

    pub fn find_func_with_name_and_trait_id(
        &self,
        func_name: &Ident,
        trait_id: &Option<TraitId>,
    ) -> Option<FuncId> {
        let (module_def, _, _) = self.values.get(func_name)?.get(trait_id)?;
        match module_def {
            ModuleDefId::FunctionId(id) => Some(*id),
            _ => None,
        }
    }

    pub fn find_name(&self, name: &Ident) -> PerNs {
        // Names, not associated with traits are searched first. If not found, we search for name, coming from a trait.
        // If we find only one name from trait, we return it. If there are multiple traits, providing the same name, we return None.
        let find_name_in = |a: &HashMap<Ident, Scope>| {
            if let Some(t) = a.get(name) {
                if let Some(tt) = t.get(&None) {
                    Some(*tt)
                } else if t.len() == 1 {
                    t.values().last().cloned()
                } else {
                    None
                }
            } else {
                None
            }
        };

        PerNs { types: find_name_in(&self.types), values: find_name_in(&self.values) }
    }

    pub fn find_name_for_trait_id(&self, name: &Ident, trait_id: &Option<TraitId>) -> PerNs {
        PerNs {
            types: if let Some(t) = self.types.get(name) { t.get(trait_id).cloned() } else { None },
            values: if let Some(v) = self.values.get(name) {
                v.get(trait_id).cloned()
            } else {
                None
            },
        }
    }

    pub fn names(&self) -> impl Iterator<Item = &Ident> {
        self.types.keys().chain(self.values.keys())
    }

    pub fn definitions(&self) -> Vec<ModuleDefId> {
        self.defs.clone()
    }

    pub fn types(&self) -> &HashMap<Ident, Scope> {
        &self.types
    }

    pub fn values(&self) -> &HashMap<Ident, Scope> {
        &self.values
    }
}
