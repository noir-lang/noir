use super::{namespace::PerNs, ModuleDefId, ModuleId};
use crate::{
    node_interner::{FuncId, TraitId},
    Ident,
};
use std::collections::{hash_map::Entry, HashMap};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Visibility {
    Public,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct ItemScope {
    types: HashMap<Ident, HashMap<Option<TraitId>, (ModuleDefId, Visibility)>>,
    values: HashMap<Ident, HashMap<Option<TraitId>, (ModuleDefId, Visibility)>>,

    defs: Vec<ModuleDefId>,
}

impl ItemScope {
    pub fn add_definition(
        &mut self,
        name: Ident,
        mod_def: ModuleDefId,
        trait_id: Option<TraitId>,
    ) -> Result<(), (Ident, Ident)> {
        self.add_item_to_namespace(name, mod_def, trait_id)?;
        self.defs.push(mod_def);
        Ok(())
    }

    /// Returns an Err if there is already an item
    /// in the namespace with that exact name.
    /// The Err will return (old_item, new_item)
    pub fn add_item_to_namespace(
        &mut self,
        name: Ident,
        mod_def: ModuleDefId,
        trait_id: Option<TraitId>,
    ) -> Result<(), (Ident, Ident)> {
        let add_item =
            |map: &mut HashMap<Ident, HashMap<Option<TraitId>, (ModuleDefId, Visibility)>>| {
                if let Entry::Occupied(mut o) = map.entry(name.clone()) {
                    let trait_hashmap = o.get_mut();
                    if let Entry::Occupied(_) = trait_hashmap.entry(trait_id) {
                        let old_ident = o.key();
                        Err((old_ident.clone(), name))
                    } else {
                        trait_hashmap.insert(trait_id, (mod_def, Visibility::Public));
                        Ok(())
                    }
                } else {
                    let mut trait_hashmap = HashMap::new();
                    trait_hashmap.insert(trait_id, (mod_def, Visibility::Public));
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
            ModuleDefId::GlobalId(_) => add_item(&mut self.values),
        }
    }

    pub fn find_module_with_name(&self, mod_name: &Ident) -> Option<&ModuleId> {
        let (module_def, _) = self.types.get(mod_name)?.get(&None)?;
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
            Some((module_def, _)) => match module_def {
                ModuleDefId::FunctionId(id) => Some(*id),
                _ => None,
            },
            None => {
                if trait_hashmap.len() == 1 {
                    let (module_def, _) = trait_hashmap.get(trait_hashmap.keys().last()?)?;
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
        let (module_def, _) = self.values.get(func_name)?.get(trait_id)?;
        match module_def {
            ModuleDefId::FunctionId(id) => Some(*id),
            _ => None,
        }
    }

    pub fn find_name(&self, name: &Ident) -> PerNs {
        // Names, not associated with traits are searched first. If not found, we search for name, coming from a trait.
        // If we find only one name from trait, we return it. If there are multiple traits, providing the same name, we return None.
        let find_name_in =
            |a: &HashMap<Ident, HashMap<Option<TraitId>, (ModuleDefId, Visibility)>>| {
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

    pub fn definitions(&self) -> Vec<ModuleDefId> {
        self.defs.clone()
    }

    pub fn types(&self) -> &HashMap<Ident, HashMap<Option<TraitId>, (ModuleDefId, Visibility)>> {
        &self.types
    }

    pub fn values(&self) -> &HashMap<Ident, HashMap<Option<TraitId>, (ModuleDefId, Visibility)>> {
        &self.values
    }

    pub fn remove_definition(&mut self, name: &Ident) {
        self.types.remove(name);
        self.values.remove(name);
    }
}
