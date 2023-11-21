use crate::hir_def::types::Type;
pub use noirc_errors::debug_info::{Types, VariableTypes, Variables};
use noirc_printable_type::PrintableType;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct DebugTypes {
    variables: HashMap<u32, (String, u32)>, // var_id => (var_name, type_id)
    types: HashMap<PrintableType, u32>,
    next_type_id: u32,
}

impl DebugTypes {
    pub fn insert_var(&mut self, var_id: u32, var_name: &str, var_type: Type) {
        let ptype: PrintableType = var_type.into();
        let type_id = self.types.get(&ptype).cloned().unwrap_or_else(|| {
            let type_id = self.next_type_id;
            self.next_type_id += 1;
            self.types.insert(ptype, type_id);
            type_id
        });
        self.variables.insert(var_id, (var_name.to_string(), type_id));
    }
}

impl Into<VariableTypes> for DebugTypes {
    fn into(self) -> VariableTypes {
        (
            self.variables.into_iter().collect(),
            self.types.into_iter().map(|(ptype, type_id)| (type_id, ptype)).collect(),
        )
    }
}
