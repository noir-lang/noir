use std::collections::{HashMap,HashSet};
use acvm::acir::brillig::Value;

#[derive(Debug, Default, Clone)]
pub struct DebugVars {
    id_to_name: HashMap<u32,String>,
    active: HashSet<u32>,
    id_to_value: HashMap<u32, Value>, // TODO: something more sophisticated for lexical levels
}

impl DebugVars {
    pub fn new(vars: &HashMap<u32, String>) -> Self {
        let mut debug_vars = Self::default();
        debug_vars.id_to_name = vars.clone();
        debug_vars
    }

    pub fn get_values<'a>(&'a self) -> HashMap<&'a str, &'a Value> {
        self.active.iter().filter_map(|var_id| {
            self.id_to_name.get(var_id).and_then(|name| {
                self.id_to_value.get(var_id).map(|value| (name.as_str(), value))
            })
        }).collect()
    }

    pub fn insert_variables(&mut self, vars: &HashMap<u32,String>) {
        vars.iter().for_each(|(var_id, var_name)| {
            self.id_to_name.insert(*var_id, var_name.clone());
        });
    }

    pub fn set(&mut self, var_id: u32, value: Value) {
        let name = self.id_to_name.get(&var_id).unwrap();
        println!["\n\n######## SET {name}[{var_id}] = {value:?}\n"];
        self.active.insert(var_id);
        self.id_to_value.insert(var_id, value);
    }

    pub fn get<'a>(&'a mut self, var_id: u32) -> Option<&'a Value> {
        self.id_to_value.get(&var_id)
    }

    pub fn drop(&mut self, var_id: u32) {
        let name = self.id_to_name.get(&var_id).unwrap();
        println!["\n\n######## DROP {name}[{var_id}]\n"];
        self.active.remove(&var_id);
    }
}
