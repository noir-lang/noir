//! Defines [ComptimeScopes], the local variables of the comptime code being interpreted.
use rustc_hash::FxHashMap as HashMap;

use crate::node_interner::DefinitionId;

use super::Value;

/// The comptime local variables in scope, one map per scope with the innermost scope last.
///
/// The stack spans every comptime function currently being interpreted, not just the innermost
/// one. Rather than physically removing a caller's scopes when a function is entered, the
/// `floor` is raised to the top of the stack so that only the callee's own scopes are visible.
/// This keeps entering a function O(1) regardless of how deep the call stack is.
///
/// The runtime elaborator pushes a scope here for every block it enters as well, so that a
/// `comptime let` in a runtime block is dropped when the block ends.
///
/// Globals are not stored here: their values live in the
/// `GlobalInfo` of each global in the [`NodeInterner`](crate::node_interner::NodeInterner).
#[derive(Debug)]
pub(crate) struct ComptimeScopes {
    /// Never empty: the bottom scope holds variables defined outside of any block and is
    /// never popped.
    scopes: Vec<HashMap<DefinitionId, Value>>,

    /// Index into `scopes` of the first scope visible to the function currently being
    /// interpreted. Scopes below it belong to enclosing callers.
    floor: usize,
}

impl Default for ComptimeScopes {
    fn default() -> Self {
        Self { scopes: vec![HashMap::default()], floor: 0 }
    }
}

impl ComptimeScopes {
    pub(crate) fn push(&mut self) {
        self.scopes.push(HashMap::default());
    }

    /// Pops the innermost scope.
    ///
    /// Panics if that would pop the bottom scope.
    pub(crate) fn pop(&mut self) {
        assert!(self.scopes.len() > 1, "Cannot pop the bottom comptime scope");
        self.scopes.pop();
    }

    /// Hides every scope currently on the stack and pushes a scope for the callee's
    /// parameters. Returns the previous floor, to be given back to [`Self::exit_function`].
    pub(crate) fn enter_function(&mut self) -> usize {
        let previous_floor = std::mem::replace(&mut self.floor, self.scopes.len());
        self.push();
        previous_floor
    }

    /// Drops every scope pushed since the matching [`Self::enter_function`] and makes the
    /// caller's scopes visible again.
    pub(crate) fn exit_function(&mut self, previous_floor: usize) {
        self.scopes.truncate(self.floor);
        self.floor = previous_floor;
    }

    /// Defines `id` in the innermost scope.
    pub(crate) fn define(&mut self, id: DefinitionId, value: Value) {
        self.scopes.last_mut().expect("There is always a bottom scope").insert(id, value);
    }

    /// The value of `id` in the innermost visible scope that defines it.
    pub(crate) fn get(&self, id: DefinitionId) -> Option<&Value> {
        self.scopes[self.floor..].iter().rev().find_map(|scope| scope.get(&id))
    }

    /// The value of `id` in the innermost visible scope that defines it.
    pub(crate) fn get_mut(&mut self, id: DefinitionId) -> Option<&mut Value> {
        self.scopes[self.floor..].iter_mut().rev().find_map(|scope| scope.get_mut(&id))
    }

    /// The variables visible to the function currently being interpreted, outermost scope
    /// first.
    pub(crate) fn visible_scopes(&self) -> &[HashMap<DefinitionId, Value>] {
        &self.scopes[self.floor..]
    }
}
