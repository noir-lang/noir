use std::collections::HashMap;

use crate::node_interner::FuncId;

use super::{namespace::PerNs, ModuleDefId, ModuleId};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Visibility {
    Public,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct ItemScope {
    types: HashMap<String, (ModuleDefId, Visibility)>,
    values: HashMap<String, (ModuleDefId, Visibility)>,

    defs: Vec<ModuleDefId>,
}

impl ItemScope {
    pub fn add_definition(&mut self, name: String, mod_def: ModuleDefId) {
        self.add_item_to_namespace(name, mod_def).unwrap();
        self.defs.push(mod_def);
    }

    pub fn add_item_to_namespace(
        &mut self,
        name: String,
        mod_def: ModuleDefId,
    ) -> Result<(), String> {
        let old_value = match &mod_def {
            ModuleDefId::ModuleId(_) => self
                .types
                .insert(name.clone(), (mod_def, Visibility::Public)),
            ModuleDefId::FunctionId(_) => self
                .values
                .insert(name.clone(), (mod_def, Visibility::Public)),
        };
        match old_value {
            None => Ok(()),
            Some(_) => {
                // XXX: If a module has the same function name twice, this error will trigger or module def.
                // Not an ice, but a user defined error
                Err(name)
            }
        }
    }

    pub fn define_module_def(&mut self, name: String, mod_id: ModuleId) {
        self.add_definition(name, mod_id.into())
    }
    pub fn define_func_def(&mut self, name: String, local_id: FuncId) {
        self.add_definition(name, local_id.into())
    }

    pub fn find_module_with_name(&self, mod_name: &str) -> Option<&ModuleId> {
        let (module_def, _) = self.types.get(mod_name)?;
        match module_def {
            ModuleDefId::ModuleId(id) => Some(id),
            _ => None,
        }
    }
    pub fn find_func_with_name(&self, func_name: &str) -> Option<FuncId> {
        let (module_def, _) = self.values.get(func_name)?;
        match module_def {
            ModuleDefId::FunctionId(id) => Some(*id),
            _ => None,
        }
    }
    pub fn find_name(&self, name: &str) -> PerNs {
        PerNs {
            types: self.types.get(name).cloned(),
            values: self.values.get(name).cloned(),
        }
    }

    pub fn definitions(&self) -> Vec<ModuleDefId> {
        self.defs.clone()
    }
    pub fn types(&self) -> &HashMap<String, (ModuleDefId, Visibility)> {
        &self.types
    }
    pub fn values(&self) -> &HashMap<String, (ModuleDefId, Visibility)> {
        &self.values
    }
}
