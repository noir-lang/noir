use crate::hir_def::types::Type;
pub use noirc_errors::debug_info::{Types, VariableTypes, Variables};
use noirc_printable_type::PrintableType;
use std::collections::HashMap;

/// We keep a collection of the debug variables and their types in this
/// structure. The fe_var_id refers to the ID given when inserting the
/// instrumentation probe. This variable does not have a type yet and hence it
/// can be instantiated multiple types if it's in the context of a generic
/// variable. The var_id refers to the ID of the instantiated variable which
/// will have a valid type.
#[derive(Debug, Clone, Default)]
pub struct DebugTypes {
    fe_to_vars: HashMap<u32, u32>,          // fe_var_id => var_id
    variables: HashMap<u32, (String, u32)>, // var_id => (var_name, type_id)
    types: HashMap<PrintableType, u32>,
    id_to_type: HashMap<u32, PrintableType>,
    next_type_id: u32,
    next_var_id: u32,
}

impl DebugTypes {
    pub fn insert_var(&mut self, fe_var_id: u32, var_name: &str, var_type: Type) -> u32 {
        let ptype: PrintableType = var_type.follow_bindings().into();
        let type_id = self.types.get(&ptype).cloned().unwrap_or_else(|| {
            let type_id = self.next_type_id;
            self.next_type_id += 1;
            self.types.insert(ptype.clone(), type_id);
            self.id_to_type.insert(type_id, ptype);
            type_id
        });
        let existing_var_id = self.fe_to_vars.get(&fe_var_id).and_then(|var_id| {
            let (_, existing_type_id) = self.variables.get(var_id).unwrap();
            if *existing_type_id == type_id {
                Some(var_id)
            } else {
                None
            }
        });
        if let Some(var_id) = existing_var_id {
            *var_id
        } else {
            let var_id = self.next_var_id;
            self.next_var_id += 1;
            self.variables.insert(var_id, (var_name.to_string(), type_id));
            self.fe_to_vars.insert(fe_var_id, var_id);
            var_id
        }
    }

    pub fn get_var_id(&self, fe_var_id: u32) -> Option<u32> {
        self.fe_to_vars.get(&fe_var_id).copied()
    }

    pub fn get_type(&self, fe_var_id: u32) -> Option<&PrintableType> {
        self.fe_to_vars
            .get(&fe_var_id)
            .and_then(|var_id| self.variables.get(var_id))
            .and_then(|(_, type_id)| self.id_to_type.get(type_id))
    }
}

impl From<DebugTypes> for VariableTypes {
    fn from(val: DebugTypes) -> Self {
        (
            val.variables.into_iter().collect(),
            val.types.into_iter().map(|(ptype, type_id)| (type_id, ptype)).collect(),
        )
    }
}
