use crate::{TYPE_RECURSION_LIMIT, Type, node_interner::TypeId};

/// A type to prevent infinite recursion while traversing types recursively.
#[derive(Clone, Default)]
pub(crate) struct TypeRecursionContext {
    depth: u32,
    type_recursion_context: im::HashSet<(TypeId, Vec<Type>)>,
}

impl TypeRecursionContext {
    /// Increases the recursion depth.
    pub(crate) fn recur(mut self) -> Self {
        if self.depth >= TYPE_RECURSION_LIMIT {
            panic!("Types are too deep!");
        }
        self.depth += 1;
        self
    }

    /// Tracks a data type and its generics. Returns whether the data type wasn't already being tracked.
    pub(crate) fn insert_data_type(&mut self, data_type_id: TypeId, generics: Vec<Type>) -> bool {
        self.type_recursion_context.insert((data_type_id, generics)).is_none()
    }
}
