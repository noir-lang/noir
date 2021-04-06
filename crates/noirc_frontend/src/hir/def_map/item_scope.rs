use super::{namespace::PerNs, ModuleDefId, ModuleId};
use crate::{node_interner::FuncId, Ident};
use std::collections::{hash_map::Entry, HashMap};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Visibility {
    Public,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct ItemScope {
    types: HashMap<Ident, (ModuleDefId, Visibility)>,
    values: HashMap<Ident, (ModuleDefId, Visibility)>,

    defs: Vec<ModuleDefId>,
}

impl ItemScope {
    pub fn add_definition(
        &mut self,
        name: Ident,
        mod_def: ModuleDefId,
    ) -> Result<(), (Ident, Ident)> {
        self.add_item_to_namespace(name, mod_def)?;
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
    ) -> Result<(), (Ident, Ident)> {
        match &mod_def {
            ModuleDefId::ModuleId(_) => {
                if let Entry::Occupied(o) = self.types.entry(name.clone()) {
                    let old_ident = o.key();
                    return Err((old_ident.clone(), name));
                }

                self.types.insert(name, (mod_def, Visibility::Public))
            }
            ModuleDefId::FunctionId(_) => {
                if let Entry::Occupied(o) = self.values.entry(name.clone()) {
                    let old_ident = o.key();
                    return Err((old_ident.clone(), name));
                }

                self.values.insert(name, (mod_def, Visibility::Public))
            }
        };
        Ok(())
    }

    pub fn define_module_def(
        &mut self,
        name: Ident,
        mod_id: ModuleId,
    ) -> Result<(), (Ident, Ident)> {
        self.add_definition(name, mod_id.into())
    }
    pub fn define_func_def(&mut self, name: Ident, local_id: FuncId) -> Result<(), (Ident, Ident)> {
        self.add_definition(name, local_id.into())
    }

    pub fn find_module_with_name(&self, mod_name: &Ident) -> Option<&ModuleId> {
        let (module_def, _) = self.types.get(mod_name)?;
        match module_def {
            ModuleDefId::ModuleId(id) => Some(id),
            _ => None,
        }
    }
    pub fn find_func_with_name(&self, func_name: &Ident) -> Option<FuncId> {
        let (module_def, _) = self.values.get(func_name)?;
        match module_def {
            ModuleDefId::FunctionId(id) => Some(*id),
            _ => None,
        }
    }
    pub fn find_name(&self, name: &Ident) -> PerNs {
        PerNs {
            types: self.types.get(name).cloned(),
            values: self.values.get(name).cloned(),
        }
    }

    pub fn definitions(&self) -> Vec<ModuleDefId> {
        self.defs.clone()
    }
    pub fn types(&self) -> &HashMap<Ident, (ModuleDefId, Visibility)> {
        &self.types
    }
    pub fn values(&self) -> &HashMap<Ident, (ModuleDefId, Visibility)> {
        &self.values
    }
}
