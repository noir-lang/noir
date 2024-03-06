use super::ModuleDefId;
use crate::ModuleVisibility;

// This works exactly the same as in r-a, just simplified
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct PerNs {
    pub types: Option<(ModuleDefId, ModuleVisibility, bool)>,
    pub values: Option<(ModuleDefId, ModuleVisibility, bool)>,
}

impl PerNs {
    pub fn types(t: ModuleDefId) -> PerNs {
        PerNs { types: Some((t, ModuleVisibility::Public, false)), values: None }
    }

    pub fn take_types(self) -> Option<ModuleDefId> {
        self.types.map(|it| it.0)
    }

    pub fn take_values(self) -> Option<ModuleDefId> {
        self.values.map(|it| it.0)
    }

    pub fn iter_defs(self) -> impl Iterator<Item = ModuleDefId> {
        self.types.map(|it| it.0).into_iter().chain(self.values.map(|it| it.0))
    }

    pub fn iter_items(self) -> impl Iterator<Item = (ModuleDefId, ModuleVisibility, bool)> {
        self.types.into_iter().chain(self.values)
    }

    pub fn is_none(&self) -> bool {
        self.types.is_none() && self.values.is_none()
    }
}
