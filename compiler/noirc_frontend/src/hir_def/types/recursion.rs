use crate::{
    TYPE_RECURSION_LIMIT, Type,
    node_interner::{TypeAliasId, TypeId},
};

/// A type to prevent infinite recursion while traversing types recursively.
#[derive(Clone, Default)]
pub(crate) struct TypeRecursionContext {
    depth: u32,
    data_types: im::HashSet<(TypeId, Vec<Type>)>,
    aliases: im::HashSet<(TypeAliasId, Vec<Type>)>,
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
        self.data_types.insert((data_type_id, generics)).is_none()
    }

    /// Tracks an alias and its generics. Returns whether the alias wasn't already being tracked.
    pub(crate) fn insert_alias(&mut self, alias_id: TypeAliasId, generics: Vec<Type>) -> bool {
        self.aliases.insert((alias_id, generics)).is_none()
    }
}
