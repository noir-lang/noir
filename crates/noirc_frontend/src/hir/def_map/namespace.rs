use super::{item_scope::Visibility, ModuleDefId};

// This works exactly the same as in r-a, just simplified
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct PerNs {
    pub types: Option<(ModuleDefId, Visibility)>,
    pub values: Option<(ModuleDefId, Visibility)>,
}

impl PerNs {
    pub fn types(t: ModuleDefId) -> PerNs {
        PerNs {
            types: Some((t, Visibility::Public)),
            values: None,
        }
    }

    pub fn take_types(self) -> Option<ModuleDefId> {
        self.types.map(|it| it.0)
    }

    pub fn take_values(self) -> Option<ModuleDefId> {
        self.values.map(|it| it.0)
    }

    pub fn iter_defs(self) -> impl Iterator<Item = ModuleDefId> {
        self.types
            .map(|it| it.0)
            .into_iter()
            .chain(self.values.map(|it| it.0).into_iter())
    }

    pub fn iter_items(self) -> impl Iterator<Item = (ModuleDefId, Visibility)> {
        self.types.into_iter().chain(self.values.into_iter())
    }

    pub fn is_none(&self) -> bool {
        self.types.is_none() && self.values.is_none()
    }
}
