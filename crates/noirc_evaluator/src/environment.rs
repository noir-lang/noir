use super::errors::RuntimeErrorKind;
use super::object::{Array, Object};
use acvm::acir::native_types::Witness;
use noirc_frontend::hir::scope::{
    ScopeForest as GenericScopeForest, ScopeTree as GenericScopeTree,
};

type ScopeTree = GenericScopeTree<String, Object>;
type ScopeForest = GenericScopeForest<String, Object>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuncContext {
    Main,
    NonMain,
}

pub struct Environment {
    pub func_context: FuncContext,
    env: ScopeForest,
}

impl Environment {
    /// Create a new environment, passing in a boolean flag
    /// to indicate whether this environment is for the main function
    /// or in the context of the main function. The latter would be
    /// the case, if we had closures and we need to possibly create and
    /// extend an environment.
    ///
    /// This flag is used because there are some functions which should only be
    /// callable within the main context.
    pub fn new(func_context: FuncContext) -> Environment {
        Environment { func_context, env: ScopeForest::new() }
    }

    pub fn start_function_environment(&mut self) {
        self.env.start_function()
    }
    pub fn end_function_environment(&mut self) -> ScopeTree {
        self.env.end_function()
    }

    pub fn start_scope(&mut self) {
        self.env.start_scope()
    }

    pub fn end_scope(&mut self) {
        self.env.end_scope();
    }

    pub fn store(&mut self, name: String, object: Object, is_global: bool) {
        let global_scope = self.env.get_global_scope();
        if is_global {
            println!(
                "storing global const object, name: {:?}, object: {:?}",
                name.clone(),
                object.clone()
            );
            global_scope.add_key_value(name, object);
            return;
        };
        let scope = self.env.get_mut_scope();
        scope.add_key_value(name, object);
    }

    pub fn get(&mut self, name: &str) -> Object {
        let global_scope = self.env.get_global_scope();
        let map = global_scope.0.clone();
        println!("GLOBAL SCOPE");
        for (key, value) in map {
            println!("key: {:?}, val: {:?}", key, value);
        }

        if let Some(global_name) = global_scope.find(name) {
            println!("ENV GET global_name: {:?}", global_name);
            global_name.clone()
        } else {
            let scope = self.env.current_scope_tree();
            println!("ENV GET name: {:?}", name);
            scope.find(name).unwrap().clone()
        }
    }

    // This method is somewhat of a hack, due to the fact that we do not map
    // Witness indices to variable names.
    pub fn find_with_value(&mut self, val: &Witness) -> Option<String> {
        let mut found = None;
        for scope in self.env.current_scope_tree().0.iter().rev() {
            found = scope.0.iter().find_map(|(k, v)| match v {
                Object::Null | Object::Array(_) | Object::Constants(_) | Object::Arithmetic(_) => {
                    None
                }
                Object::Integer(x) => {
                    // Integers are assumed to always be unit
                    (&x.witness == val).then(|| k)
                }
                Object::Linear(x) => (x.is_unit() && &x.witness == val).then(|| k),
            });
            if found.is_some() {
                break;
            }
        }
        found.cloned()
    }

    pub fn get_array(&mut self, name: &str) -> Result<Array, RuntimeErrorKind> {
        let poly = self.get(name);

        match poly {
            Object::Array(arr) => Ok(arr),
            k => Err(RuntimeErrorKind::ArrayNotFound {
                name: name.to_owned(),
                found_type: k.r#type().to_owned(),
            }),
        }
    }
}
