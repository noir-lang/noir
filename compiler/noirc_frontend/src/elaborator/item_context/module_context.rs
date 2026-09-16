//! The part of an item's context that says which item is being elaborated, and where it lives.

use crate::{
    hir::def_map::{LocalModuleId, ModuleId},
    node_interner::DependencyId,
};

/// Which item the elaborator is working on and which module it was written in.
///
/// The module is what unqualified paths in the item resolve against, so it moves with the item:
/// an item elaborated on demand from the middle of another one is resolved in its own module, not
/// in the module of whatever mentioned it.
pub(crate) struct ModuleContext {
    /// The module the item was written in, which every item that resolves a path has.
    local_module: LocalModuleId,

    /// The current dependency item we're resolving.
    /// Used to link items to their dependencies in the dependency graph
    current_item: Option<DependencyId>,

    /// When set, visibility checks during path resolution use this module
    /// instead of the default importing module.
    ///
    /// Set when resolving an expression on behalf of comptime code from another module (see
    /// `Expr::resolve`), so that the item resolved that way is held to the caller's visibility
    /// rather than to that of the scope it is resolved in.
    caller_module: Option<ModuleId>,
}

impl ModuleContext {
    /// Elaborating `current_item`, which was written in `local_module`.
    pub(crate) fn of_item(local_module: LocalModuleId, current_item: DependencyId) -> Self {
        Self { local_module, current_item: Some(current_item), caller_module: None }
    }

    /// Elaborating in `local_module` without naming an item, for work that belongs to no single
    /// item and so should register no dependencies, such as resolving an impl's header.
    pub(crate) fn in_module(local_module: LocalModuleId) -> Self {
        Self { local_module, current_item: None, caller_module: None }
    }

    /// Elaborating `current_item`, which is written inside the item this context describes: a
    /// trait's method, say. The module is the enclosing item's.
    pub(crate) fn nested_item(&self, current_item: DependencyId) -> Self {
        Self {
            local_module: self.local_module,
            current_item: Some(current_item),
            caller_module: None,
        }
    }

    /// The module the item was written in.
    pub(crate) fn local_module(&self) -> LocalModuleId {
        self.local_module
    }

    /// Whether the item being elaborated was written in `module`.
    pub(crate) fn is_in_module(&self, module: LocalModuleId) -> bool {
        self.local_module == module
    }

    /// Resolves the rest of the item in `module`, returning the module it replaces so the caller
    /// can put it back with [`Self::set_local_module`].
    #[must_use]
    pub(crate) fn replace_local_module(&mut self, module: LocalModuleId) -> LocalModuleId {
        std::mem::replace(&mut self.local_module, module)
    }

    pub(crate) fn set_local_module(&mut self, module: LocalModuleId) {
        self.local_module = module;
    }

    pub(crate) fn current_item(&self) -> Option<DependencyId> {
        self.current_item
    }

    pub(crate) fn set_current_item(&mut self, current_item: Option<DependencyId>) {
        self.current_item = current_item;
    }

    pub(crate) fn caller_module(&self) -> Option<ModuleId> {
        self.caller_module
    }

    pub(crate) fn set_caller_module(&mut self, caller_module: Option<ModuleId>) {
        self.caller_module = caller_module;
    }
}
