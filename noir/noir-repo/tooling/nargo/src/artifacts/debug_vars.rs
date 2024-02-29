use acvm::brillig_vm::brillig::Value;
use noirc_errors::debug_info::{
    DebugTypeId, DebugTypes, DebugVarId, DebugVariable, DebugVariables,
};
use noirc_printable_type::{decode_value, PrintableType, PrintableValue};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Clone)]
pub struct DebugVars {
    variables: HashMap<DebugVarId, DebugVariable>,
    types: HashMap<DebugTypeId, PrintableType>,
    active: HashSet<DebugVarId>,
    values: HashMap<DebugVarId, PrintableValue>,
}

impl DebugVars {
    pub fn get_variables(&self) -> Vec<(&str, &PrintableValue, &PrintableType)> {
        self.active
            .iter()
            .filter_map(|var_id| {
                self.variables.get(var_id).and_then(|debug_var| {
                    let Some(value) = self.values.get(var_id) else {
                        return None;
                    };
                    let Some(ptype) = self.types.get(&debug_var.debug_type_id) else {
                        return None;
                    };
                    Some((debug_var.name.as_str(), value, ptype))
                })
            })
            .collect()
    }

    pub fn insert_variables(&mut self, vars: &DebugVariables) {
        self.variables.extend(vars.clone());
    }

    pub fn insert_types(&mut self, types: &DebugTypes) {
        self.types.extend(types.clone());
    }

    pub fn assign_var(&mut self, var_id: DebugVarId, values: &[Value]) {
        self.active.insert(var_id);
        let type_id = &self.variables.get(&var_id).unwrap().debug_type_id;
        let ptype = self.types.get(type_id).unwrap();
        self.values.insert(var_id, decode_value(&mut values.iter().map(|v| v.to_field()), ptype));
    }

    pub fn assign_field(&mut self, var_id: DebugVarId, indexes: Vec<u32>, values: &[Value]) {
        let mut cursor: &mut PrintableValue = self
            .values
            .get_mut(&var_id)
            .unwrap_or_else(|| panic!("value unavailable for var_id {var_id:?}"));
        let cursor_type_id = &self
            .variables
            .get(&var_id)
            .unwrap_or_else(|| panic!("variable {var_id:?} not found"))
            .debug_type_id;
        let mut cursor_type = self
            .types
            .get(cursor_type_id)
            .unwrap_or_else(|| panic!("type unavailable for type id {cursor_type_id:?}"));
        for index in indexes.iter() {
            (cursor, cursor_type) = match (cursor, cursor_type) {
                (PrintableValue::Vec(array), PrintableType::Array { length, typ }) => {
                    if let Some(len) = length {
                        if *index as u64 >= *len {
                            panic!("unexpected field index past array length")
                        }
                        if *len != array.len() as u64 {
                            panic!("type/array length mismatch")
                        }
                    }
                    (array.get_mut(*index as usize).unwrap(), &*Box::leak(typ.clone()))
                }
                (
                    PrintableValue::Struct(field_map),
                    PrintableType::Struct { name: _name, fields },
                ) => {
                    if *index as usize >= fields.len() {
                        panic!("unexpected field index past struct field length")
                    }
                    let (key, typ) = fields.get(*index as usize).unwrap();
                    (field_map.get_mut(key).unwrap(), typ)
                }
                (PrintableValue::Vec(array), PrintableType::Tuple { types }) => {
                    if *index >= types.len() as u32 {
                        panic!(
                            "unexpected field index ({index}) past tuple length ({})",
                            types.len()
                        );
                    }
                    if types.len() != array.len() {
                        panic!("type/array length mismatch")
                    }
                    let typ = types.get(*index as usize).unwrap();
                    (array.get_mut(*index as usize).unwrap(), typ)
                }
                _ => {
                    panic!("unexpected assign field of {cursor_type:?} type");
                }
            };
        }
        *cursor = decode_value(&mut values.iter().map(|v| v.to_field()), cursor_type);
        self.active.insert(var_id);
    }

    pub fn assign_deref(&mut self, _var_id: DebugVarId, _values: &[Value]) {
        unimplemented![]
    }

    pub fn get_type(&self, var_id: DebugVarId) -> Option<&PrintableType> {
        self.variables.get(&var_id).and_then(|debug_var| self.types.get(&debug_var.debug_type_id))
    }

    pub fn drop_var(&mut self, var_id: DebugVarId) {
        self.active.remove(&var_id);
    }
}
