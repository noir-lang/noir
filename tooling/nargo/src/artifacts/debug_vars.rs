use acvm::acir::brillig::Value;
use noirc_printable_type::PrintableType;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Clone)]
pub struct DebugVars {
    id_to_name: HashMap<u32, String>,
    active: HashSet<u32>,
    id_to_value: HashMap<u32, Value>, // TODO: something more sophisticated for lexical levels
    id_to_type: HashMap<u32, u32>,
    types: HashMap<u32, PrintableType>,
}

impl DebugVars {
    pub fn new(vars: &HashMap<u32, String>) -> Self {
        let mut debug_vars = Self::default();
        debug_vars.id_to_name = vars.clone();
        debug_vars
    }

    pub fn get_variables<'a>(&'a self) -> Vec<(&'a str, &'a Value, &'a PrintableType)> {
        self.active
            .iter()
            .filter_map(|var_id| {
                self.id_to_name
                    .get(var_id)
                    .and_then(|name| self.id_to_value.get(var_id).map(|value| (name, value)))
                    .and_then(|(name, value)| {
                        self.id_to_type.get(var_id).map(|type_id| (name, value, type_id))
                    })
                    .and_then(|(name, value, type_id)| {
                        self.types.get(type_id).map(|ptype| (name.as_str(), value, ptype))
                    })
            })
            .collect()
    }

    pub fn insert_variables(&mut self, vars: &HashMap<u32, (String, u32)>) {
        vars.iter().for_each(|(var_id, (var_name, var_type))| {
            self.id_to_name.insert(*var_id, var_name.clone());
            self.id_to_type.insert(*var_id, *var_type);
        });
    }

    pub fn insert_types(&mut self, types: &HashMap<u32, PrintableType>) {
        types.iter().for_each(|(type_id, ptype)| {
            self.types.insert(*type_id, ptype.clone());
        });
    }

    pub fn assign(&mut self, var_id: u32, value: Value) {
        self.active.insert(var_id);
        self.id_to_value.insert(var_id, value);
    }

    pub fn assign_member(&mut self, _var_id: u32, _member_id: u32, _value: Value) {
        // TODO
    }

    pub fn assign_index(&mut self, _var_id: u32, _index: u64, _value: Value) {
        // TODO
    }

    pub fn assign_deref(&mut self, _var_id: u32, _value: Value) {
        // TODO
    }

    pub fn get<'a>(&'a mut self, var_id: u32) -> Option<&'a Value> {
        self.id_to_value.get(&var_id)
    }

    pub fn drop(&mut self, var_id: u32) {
        self.active.remove(&var_id);
    }
}
