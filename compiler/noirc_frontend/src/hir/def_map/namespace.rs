use super::{ModuleDefId, NamespaceItem};
use crate::ast::ItemVisibility;

/// The namespace an item lives in. Noir resolves types and values independently, so a
/// type-namespace item and a value-namespace item can legally share a name within a module
/// (e.g. `struct N` and `fn N`).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Namespace {
    Type,
    Value,
}

/// Result of looking up a name in type and value definitions in scope.
///
/// This works exactly the same as in r-a, just simplified.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Default)]
pub struct PerNs {
    pub types: Option<NamespaceItem>,
    pub values: Option<NamespaceItem>,
}

impl PerNs {
    /// Creates a [`PerNs`] with a public [`ModuleDefId`] in `types`, and no `values`.
    pub fn types(t: ModuleDefId) -> PerNs {
        PerNs {
            types: Some(NamespaceItem {
                id: t,
                visibility: ItemVisibility::Public,
                is_prelude: false,
            }),
            values: None,
        }
    }

    /// Iterate the results in both `types` and `values`.
    pub fn iter_items(self) -> impl Iterator<Item = NamespaceItem> {
        self.types.into_iter().chain(self.values)
    }

    /// Returns `true` if both `types` and `values` are empty.
    pub fn is_none(&self) -> bool {
        self.types.is_none() && self.values.is_none()
    }
}
