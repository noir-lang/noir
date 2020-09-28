use super::polynomial::{Polynomial, Array};
use std::collections::HashMap;

pub struct Environment(pub HashMap<String, Polynomial>);

impl Environment {
    pub fn new() -> Environment {
        Environment(HashMap::new())
    }
    pub fn store(&mut self, name: String, poly: Polynomial) {
        self.0.insert(name, poly);
    }
    pub fn get(&mut self, name: String) -> Polynomial {
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
            Polynomial::Array(arr) => arr,
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
