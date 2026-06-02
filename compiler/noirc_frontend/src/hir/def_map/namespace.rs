use super::ModuleDefId;
use crate::ast::ItemVisibility;

/// Result of looking up a name in type and value definitions in scope.
///
/// This works exactly the same as in r-a, just simplified.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Default)]
pub struct PerNs {
    pub types: Option<(ModuleDefId, ItemVisibility, bool)>,
    pub values: Option<(ModuleDefId, ItemVisibility, bool)>,
}

impl PerNs {
    /// Creates a [PerNs] with a public [ModuleDefId] in `types`, and no `values`.
    pub fn types(t: ModuleDefId) -> PerNs {
        PerNs { types: Some((t, ItemVisibility::Public, false)), values: None }
    }

    /// Iterate the [ModuleDefId]s in both `types` and `values`.
    pub fn iter_defs(self) -> impl Iterator<Item = ModuleDefId> {
        self.iter_items().map(|it| it.0)
    }

    /// Iterate the results in both `types` and `values`.
    pub fn iter_items(self) -> impl Iterator<Item = (ModuleDefId, ItemVisibility, bool)> {
        self.types.into_iter().chain(self.values)
    }

    /// Returns `true` if both `types` and `values` are empty.
    pub fn is_none(&self) -> bool {
        self.types.is_none() && self.values.is_none()
    }
}
