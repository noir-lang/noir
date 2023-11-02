use std::collections::HashMap;
use acvm::acir::brillig::Value;

#[derive(Debug, Default, Clone)]
pub struct DebugVars {
    id_to_name: HashMap<u32,String>,
    name_to_id: HashMap<String,u32>,
    id_to_value: HashMap<u32, Value>, // TODO: something more sophisticated for lexical levels
}

impl DebugVars {
    pub fn new(vars: &HashMap<String,u32>) -> Self {
        let mut debug_vars = Self::default();
        debug_vars.id_to_name = vars.iter().map(|(name,id)| (*id, name.clone())).collect();
        debug_vars.name_to_id = vars.clone();
        debug_vars
    }

    pub fn get_values<'a>(&'a self) -> HashMap<String,&'a Value> {
        self.id_to_value.iter().filter_map(|(var_id,value)| {
            self.id_to_name.get(var_id).map(|name| {
                (name.clone(),value)
            })
        }).collect()
    }

    pub fn insert_variables(&mut self, vars: &HashMap<String,u32>) {
        vars.iter().for_each(|(var_name,var_id)| {
            self.id_to_name.insert(*var_id, var_name.clone());
            self.name_to_id.insert(var_name.clone(), *var_id);
        });
    }

    pub fn set_by_id(&mut self, var_id: u32, value: Value) {
        self.id_to_value.insert(var_id, value);
    }

    pub fn get_by_id<'a>(&'a mut self, var_id: u32) -> Option<&'a Value> {
        self.id_to_value.get(&var_id)
    }

    pub fn get_by_name<'a>(&'a mut self, var_name: &str) -> Option<&'a Value> {
        self.name_to_id.get(var_name).and_then(|var_id| self.id_to_value.get(var_id))
    }
}
