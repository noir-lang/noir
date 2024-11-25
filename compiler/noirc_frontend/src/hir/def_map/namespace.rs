use super::ModuleDefId;
use crate::ast::ItemVisibility;

// This works exactly the same as in r-a, just simplified
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct PerNs {
    pub types: Option<(ModuleDefId, ItemVisibility, bool)>,
    pub values: Option<(ModuleDefId, ItemVisibility, bool)>,
}

impl PerNs {
    pub fn types(t: ModuleDefId) -> PerNs {
        PerNs { types: Some((t, ItemVisibility::Public, false)), values: None }
    }

    pub fn iter_defs(self) -> impl Iterator<Item = ModuleDefId> {
        self.types.map(|it| it.0).into_iter().chain(self.values.map(|it| it.0))
    }

    pub fn iter_items(self) -> impl Iterator<Item = (ModuleDefId, ItemVisibility, bool)> {
        self.types.into_iter().chain(self.values)
    }

    pub fn is_none(&self) -> bool {
        self.types.is_none() && self.values.is_none()
    }
}
