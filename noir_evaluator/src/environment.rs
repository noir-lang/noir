use super::object::{Object, Array};
use std::collections::HashMap;

pub struct Environment(pub HashMap<String, Object>);

impl Environment {
    pub fn new() -> Environment {
        Environment(HashMap::new())
    }
    pub fn store(&mut self, name: String, object: Object) {
        self.0.insert(name, object);
    }
    pub fn get(&mut self, name: String) -> Object {
        dbg!(name.clone());
        self.0.get(&name).unwrap().clone() // XXX: Fix unwrap
    }
    pub fn get_array(&mut self, name: String) -> &Array {
        dbg!(name.clone());
        let poly = match self.0.get(&name) {
            Some(poly) => poly,
            None => panic!("Cannot find variable with name {:?}",name),
        };

        match poly {
            Object::Array(arr) => arr,
            _=> panic!("Cannot find an array with that name")
        }
    }
    pub fn extend(&mut self, env: &Environment) {
        // XXX: Fix to use references
        let map = env.0.clone();
        self.0.extend(map.into_iter());
    }

    pub fn debug(&self) {
        for kv in self.0.iter() {
            dbg!(kv);
        }
    }
}
