use arbitrary::Unstructured;
use noirc_frontend::monomorphization::ast::Type;

use super::{Name, types};

pub(crate) type Variable = (/*mutable*/ bool, Name, Type);

/// A layer of variables available to choose from in blocks.
#[derive(Debug, Clone)]
pub(crate) struct Scope<K: Ord> {
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
    K: Ord + Clone + Copy + std::fmt::Debug,
{
    /// Create the initial scope from function parameters.
    pub fn new(vars: impl Iterator<Item = (K, bool, Name, Type)>) -> Self {
        let mut scope = Self { variables: im::OrdMap::new(), producers: im::OrdMap::new() };
        for (id, mutable, name, typ) in vars {
            scope.add(id, mutable, name, typ);
        }
        scope
    }

    /// Add a new variable to the scope.
    pub fn add(&mut self, id: K, mutable: bool, name: String, typ: Type) {
        for typ in types::types_produced(&typ) {
            self.producers.entry(typ).or_default().insert(id);
        }
        self.variables.insert(id, (mutable, name, typ));
    }

    /// Remove a variable from the scope.
    pub fn remove(&mut self, id: &K) {
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

    /// Check if there are any variables in scope.
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// Get a variable in scope.
    pub fn get_variable(&self, id: &K) -> &Variable {
        self.variables.get(id).unwrap_or_else(|| panic!("variable doesn't exist: {:?}", id))
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
}
