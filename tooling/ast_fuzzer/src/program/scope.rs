use arbitrary::Unstructured;
use noirc_frontend::monomorphization::ast::Type;
use std::{fmt::Debug, vec};

use super::{Name, types};

pub(crate) type Variable = (/*mutable*/ bool, Name, Type);

/// A layer of variables available to choose from in blocks.
#[derive(Debug, Clone)]
pub struct Scope<K: Ord> {
    /// ID and type of variables created in all visible scopes,
    /// which includes this scope and its ancestors.
    variables: im::OrdMap<K, Variable>,
    /// Reverse index of variables which can produce a type.
    /// For example an `(u8, [u64; 4])` can produce the tuple itself,
    /// the array in it, and both primitive types.
    producers: im::OrdMap<Type, im::OrdSet<K>>,
}

impl<K> Scope<K>
where
    K: Ord + Clone + Copy + Debug,
{
    /// Create the initial scope from function parameters.
    pub fn from_variables(vars: impl Iterator<Item = (K, bool, Name, Type)>) -> Self {
        let mut scope = Self { variables: im::OrdMap::new(), producers: im::OrdMap::new() };
        for (id, mutable, name, typ) in vars {
            scope.add(id, mutable, name, typ);
        }
        scope
    }

    /// Add a new variable to the scope.
    ///
    /// Private because:
    /// * we can't add globals
    /// * locals should go though the `ScopeStack`
    fn add(&mut self, id: K, mutable: bool, name: String, typ: Type) {
        assert!(!self.variables.contains_key(&id), "variable already exists");
        for typ in types::types_produced(&typ) {
            self.producers.entry(typ).or_default().insert(id);
        }
        self.variables.insert(id, (mutable, name, typ));
    }

    /// Remove a variable from the scope.
    ///
    /// Private because:
    /// * we can't add or remove globals
    /// * locals should go through the `ScopeStack`
    fn remove(&mut self, id: &K) {
        // Remove the variable
        if self.variables.remove(id).is_none() {
            return;
        }
        // Remove the variable from the producers of all types.
        // At the end remove types which are no longer produced.
        let mut emptied = Vec::new();
        let types = self.producers.keys().cloned().collect::<Vec<_>>();
        for typ in types {
            if let Some(ps) = self.producers.get_mut(&typ) {
                ps.remove(id);
                if ps.is_empty() {
                    emptied.push(typ);
                }
            }
        }
        for typ in emptied {
            self.producers.remove(&typ);
        }
    }

    /// Choose a random producer of a type, if there is one.
    pub fn choose_producer(
        &self,
        u: &mut Unstructured,
        typ: &Type,
    ) -> arbitrary::Result<Option<K>> {
        let Some(vs) = self.producers.get(typ) else {
            return Ok(None);
        };
        if vs.is_empty() {
            return Ok(None);
        }
        u.choose_iter(vs.iter()).map(Some).map(|v| v.cloned())
    }

    /// Choose a random producer of a type matching some criteria.
    pub(super) fn choose_producer_filtered(
        &self,
        u: &mut Unstructured,
        typ: &Type,
        pred: impl Fn(&K, &Variable) -> bool,
    ) -> arbitrary::Result<Option<K>> {
        let Some(vs) = self.producers.get(typ) else {
            return Ok(None);
        };

        let candidates = vs
            .iter()
            .filter(|id| {
                let v = self.get_variable(id);
                pred(id, v)
            })
            .collect::<Vec<_>>();

        if candidates.is_empty() {
            return Ok(None);
        }

        u.choose_iter(candidates).map(Some).map(|v| v.cloned())
    }

    /// Get a variable in scope.
    pub fn get_variable(&self, id: &K) -> &Variable {
        self.variables.get(id).unwrap_or_else(|| panic!("variable doesn't exist: {id:?}"))
    }
}

impl<K> Scope<K>
where
    K: Ord,
{
    /// Check if there are any variables in scope.
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// Iterate the variables in scope.
    pub fn variables(&self) -> impl ExactSizeIterator<Item = (&K, &Variable)> {
        self.variables.iter()
    }

    /// Iterate the IDs of the variables in scope.
    pub fn variable_ids(&self) -> impl ExactSizeIterator<Item = &K> {
        self.variables.keys()
    }

    /// Iterate the types we can produce from other variables.
    pub fn types_produced(&self) -> impl ExactSizeIterator<Item = &Type> {
        self.producers.keys()
    }
}

/// Scope stack as we exit and enter blocks
pub struct Stack<T>(Vec<T>);

impl<T: Clone> Stack<T> {
    /// Create a stack from the base layer.
    pub fn new(base: T) -> Self {
        Self(vec![base])
    }

    /// The top scope in the stack.
    pub fn current(&self) -> &T {
        self.0.last().expect("there is always the base layer")
    }

    /// The top scope in the stack.
    pub fn current_mut(&mut self) -> &mut T {
        self.0.last_mut().expect("there is always the base layer")
    }

    /// Push a new scope on top of the current one.
    pub fn enter(&mut self) {
        self.0.push(self.current().clone());
    }

    /// Remove the last layer of block variables.
    pub fn exit(&mut self) {
        self.0.pop();
        assert!(!self.0.is_empty(), "never pop the base layer");
    }

    /// Iterate over the layers, starting the base layer.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        self.0.iter_mut()
    }
}

/// Scope stack as we exit and enter blocks
pub type ScopeStack<K> = Stack<Scope<K>>;

impl<K> ScopeStack<K>
where
    K: Ord + Clone + Copy + Debug,
{
    /// Create a stack from the base variables.
    pub fn from_variables(vars: impl Iterator<Item = (K, bool, Name, Type)>) -> Self {
        Self(vec![Scope::from_variables(vars)])
    }

    /// Add a new variable to the current scope.
    pub fn add(&mut self, id: K, mutable: bool, name: String, typ: Type) {
        self.0.last_mut().expect("there is always a layer").add(id, mutable, name, typ);
    }

    /// Remove a variable from all scopes.
    pub fn remove(&mut self, id: &K) {
        for scope in self.0.iter_mut() {
            scope.remove(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use noirc_frontend::monomorphization::ast::{LocalId, Type};

    use crate::program::types;

    use super::ScopeStack;

    #[test]
    fn test_scope_stack() {
        let foo_type =
            Type::Tuple(vec![Type::Field, Type::Bool, Type::Array(4, Box::new(types::U32))]);

        let mut stack = ScopeStack::from_variables(
            [(LocalId(0), false, "foo".to_string(), foo_type)].into_iter(),
        );

        stack.enter();
        stack.add(LocalId(1), false, "bar".to_string(), Type::String(10));

        let scope0 = &stack.0[0];
        let scope1 = &stack.0[1];

        assert_eq!(scope0.variable_ids().len(), 1);
        assert_eq!(scope0.types_produced().len(), 5 + 2); // What we see plus upcasts from u32 to u64 and u128
        assert_eq!(scope1.variable_ids().len(), 2);
        assert_eq!(scope1.types_produced().len(), 5 + 2 + 1);

        stack.exit();
        assert_eq!(stack.0.len(), 1);
    }
}
