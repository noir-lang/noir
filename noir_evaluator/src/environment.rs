use super::object::{Object, Array};

use noirc_frontend::analyser::scope::{Scope as GenericScope, ScopeTree as GenericScopeTree, ScopeForest as GenericScopeForest};

type Scope = GenericScope<String, Object>;
type ScopeTree = GenericScopeTree<String, Object>;
type ScopeForest = GenericScopeForest<String, Object>;

pub struct Environment(ScopeForest);

impl Environment {
    pub fn new() -> Environment {
        Environment(ScopeForest::new())
    }

    pub fn start_function_environment(&mut self) {
        self.0.start_function()
    }
    pub fn end_function_environment(&mut self) -> ScopeTree {
        self.0.end_function()
    }

    pub fn start_for_loop(&mut self) {
        self.0.start_for_loop()
    }

    pub fn end_for_loop(&mut self) {
        self.0.end_for_loop();
    }

    pub fn store(&mut self, name: String, object: Object) {

        let scope = self.0.get_mut_scope();
        scope.add_key_value(name.clone(),object);
    }
    pub fn update(&mut self, name: String, object: Object) {

        panic!("Re-assignment feature is currently being rolled-back. Note however that you can only re-assign what is to become Field");
        let scope = self.0.current_scope_tree();
        let old_object = scope.find_key(&name).unwrap();


        *old_object = object;
    }
    pub fn get(&mut self, name: String) -> Object {
        let scope = self.0.current_scope_tree();
        scope.find_key(&name).unwrap().clone()
    }
    pub fn get_array(&mut self, name: String) -> Array {
        let poly = self.get(name);

        match poly {
            Object::Array(arr) => arr,
            _=> panic!("Cannot find an array with that name")
        }
    }
}
