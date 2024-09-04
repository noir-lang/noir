use std::collections::HashSet;

use rustc_hash::FxHashMap as HashMap;

use crate::{ast::Ident, hir::def_map::ModuleId};

#[derive(Debug, Default)]
pub struct UsageTracker {
    /// List of all unused imports in each module. Each time something is imported it's added
    /// to the module's set. When it's used, it's removed. At the end of the program only unused imports remain.
    unused_imports: HashMap<ModuleId, HashSet<Ident>>,
}

impl UsageTracker {
    pub(crate) fn add_unused_import(&mut self, module_id: ModuleId, name: Ident) {
        self.unused_imports.entry(module_id).or_default().insert(name);
    }

    pub(crate) fn mark_as_used(&mut self, current_mod_id: ModuleId, name: &Ident) {
        self.unused_imports.entry(current_mod_id).or_default().remove(name);
    }

    pub(crate) fn unused_imports(&self) -> &HashMap<ModuleId, HashSet<Ident>> {
        &self.unused_imports
    }
}
