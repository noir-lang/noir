use std::collections::HashMap;

//This needs a lot more documentation

// We could potentially use a vector of tuples (K,V) and have a enum marker to show scope beginning
/*
enum Item {
    Marker,
    Symbol(K, V)
}

struct Scope<K,V>(Vec<Item>)

We may want this because vector search is more cache friendly when there are not a lot of items.
It's not implemented yet, because nothing has been benched

*/

/// This implementation uses the terminology Scope and ScopeTree
/// A Scope is map between Keys and Values, it is possible to search for a Key in Scope, returning a mutable copy
/// A ScopeTree is a Vector of Scopes, searching for Key in ScopeTree will recursively search from the last scope until the first scope
/// to find the Key, returning None, if the Key is in None of the Scopes.
/// A ScopeTree is useful for implementing the scoping logic around for-loops, if statements and ClosureCalls.
/// Implementers will usually store a Vector of ScopeTrees to implement the logic needed for FunctionCalls
pub struct Scope<K, V>(pub HashMap<K, V>);

impl<K: std::hash::Hash + Eq + Clone, V> Scope<K, V> {
    pub fn new() -> Self {
        Scope(HashMap::with_capacity(10))
    }

    pub fn find<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq,
    {
        self.0.get_mut(key)
    }
    pub fn occupied_key(&mut self, key: &K) -> Option<&K> {
        self.0.get_key_value(key).map(|(k, _)| k)
    }

    // From HashMap: If the map did not have this key present, None is returned.
    pub fn add_key_value(&mut self, key: K, value: V) -> Option<V> {
        self.0.insert(key, value)
    }

    /// Returns an iterator over all of the elements which satisfy the predicate
    pub fn filter<F>(&self, pred: F) -> impl Iterator<Item = (&K, &V)>
    where
        F: FnMut(&(&K, &V)) -> bool,
    {
        self.0.iter().filter(pred)
    }
}

impl<K: std::hash::Hash + Eq + Clone, V> Default for Scope<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

/// ScopeTree allows one to specify that maps within the same vector are scope extensions
/// This allows one to extend the scope and then remove the extension, without affecting the unextended
/// part of the scope
pub struct ScopeTree<K, V>(pub Vec<Scope<K, V>>);

impl<K: std::hash::Hash + Eq + Clone, V> ScopeTree<K, V> {
    pub fn new() -> Self {
        let mut vec: Vec<Scope<K, V>> = Vec::with_capacity(10);
        vec.push(Scope::new());
        ScopeTree(vec)
    }

    // Returns the last pushed scope on the scope tree
    pub fn last_scope(&mut self) -> &mut Scope<K, V> {
        self.0
            .last_mut()
            .expect("Compiler Error: Tried to fetch the last scope, however no Scopes are present")
    }

    // Recursively search for a key in the scope tree
    pub fn find<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq,
    {
        for scope in self.0.iter_mut().rev() {
            if let Some(value_found) = scope.find(key) {
                return Some(value_found);
            }
        }

        None
    }

    pub fn push_scope(&mut self) {
        self.0.push(Scope::new())
    }

    pub fn pop_scope(&mut self) -> Scope<K, V> {
        self.0.pop().unwrap()
    }
}

impl<K: std::hash::Hash + Eq + Clone, V> Default for ScopeTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

// XXX: This trait is needed because when we pop off a forscope in the resolver
// We want to check it for unused variables and return. Currently,
// we only have an API for this with ScopeTree in the resolver.
impl<K: std::hash::Hash + Eq + Clone, V> From<Scope<K, V>> for ScopeTree<K, V> {
    fn from(scp: Scope<K, V>) -> Self {
        let mut tree = ScopeTree::new();
        tree.0.push(scp);
        tree
    }
}

pub struct ScopeForest<K, V> {
    pub global_scope: Scope<K, V>,
    pub scope_forest: Vec<ScopeTree<K, V>>,
}

impl<K: std::hash::Hash + Eq + Clone, V> ScopeForest<K, V> {
    pub fn new() -> ScopeForest<K, V> {
        ScopeForest { global_scope: Scope::new(), scope_forest: vec![ScopeTree::new()] }
        // ScopeForest(vec![ScopeTree::new()])
    }
    pub fn current_scope_tree(&mut self) -> &mut ScopeTree<K, V> {
        self.scope_forest
            .last_mut()
            .expect("ice: tried to fetch the current scope, however none was found")
    }

    /// Returns the last pushed scope from the current scope tree
    pub fn get_mut_scope(&mut self) -> &mut Scope<K, V> {
        self.current_scope_tree().last_scope()
    }

    fn extend_current_scope_tree(&mut self) {
        self.current_scope_tree().push_scope()
    }
    fn remove_scope_tree_extension(&mut self) -> Scope<K, V> {
        self.current_scope_tree().pop_scope()
    }
    /// Starting a function requires a new scope tree, as you do not want the functions scope to
    /// have access to the scope of the caller
    pub fn start_function(&mut self) {
        self.scope_forest.push(ScopeTree::new())
    }
    /// Ending a function requires that we removes it's whole tree of scope
    /// This is by design the current scope, which is the last element in the vector
    pub fn end_function(&mut self) -> ScopeTree<K, V> {
        self.scope_forest.pop().expect("ice: expected a scope tree, however none was found")
    }

    /// The beginning of a scope always correlates with the start of a block {}.
    /// This can be in if expressions, for loops, or functions.
    pub fn start_scope(&mut self) {
        self.extend_current_scope_tree()
    }

    /// Ends the current scope - this should correspond with the end of a BlockExpression.
    pub fn end_scope(&mut self) -> Scope<K, V> {
        self.remove_scope_tree_extension()
    }

    pub fn get_global_scope(&mut self) -> &mut Scope<K, V> {
        &mut self.global_scope
    }
}

impl<K: std::hash::Hash + Eq + Clone, V> Default for ScopeForest<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

// ScopeForest is another layer of abstraction which will handle scoping for functions
// We can have methods like: start_function() and end_function() and in the future(maybe) start_closure() end_closure()
// We will have environment use this too

// Implement for_loops in evaluator, but we need to migrate the environment to use the scope forest
