use std::collections::HashMap;

use fm::FileId;

use crate::Ident;

use super::{ItemScope, LocalModuleId};

/// Contains the actual contents of a module: its parent (if one exists),
/// children, and scope with all definitions defined within the scope.
#[derive(Debug, PartialEq, Eq)]
pub struct ModuleData {
    pub parent: Option<LocalModuleId>,
    pub children: HashMap<Ident, LocalModuleId>,
    pub scope: ItemScope,

    pub origin: ModuleOrigin,

    /// True if this module is a `contract Foo { ... }` module containing contract functions
    pub is_contract: bool,
}

impl ModuleData {
    pub fn new(
        parent: Option<LocalModuleId>,
        origin: ModuleOrigin,
        is_contract: bool,
    ) -> ModuleData {
        ModuleData {
            parent,
            children: HashMap::new(),
            scope: ItemScope::default(),
            origin,
            is_contract,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ModuleOrigin {
    CrateRoot(FileId),
    File(FileId),
}

impl ModuleOrigin {
    pub fn file_id(&self) -> FileId {
        match self {
            ModuleOrigin::CrateRoot(file_id) => *file_id,
            ModuleOrigin::File(file_id) => *file_id,
        }
    }
}

impl From<ModuleOrigin> for FileId {
    fn from(origin: ModuleOrigin) -> Self {
        origin.file_id()
    }
}

impl Default for ModuleOrigin {
    fn default() -> Self {
        ModuleOrigin::CrateRoot(FileId::default())
    }
}
