use crate::{
    ast::{Ident, ItemVisibility},
    hir::def_map::{ModuleDefId, ModuleId},
    node_interner::TraitId,
};

use super::NodeInterner;

/// Captures a reexport that happens inside a module. For example:
///
/// ```noir
/// mod moo {
/// //  ^^^ module_id
///
///   pub use foo::bar as baz;
/// //^^^ visibility      ^^^ name
/// }
/// ```
///
#[derive(Debug, Clone)]
pub struct Reexport {
    pub module_id: ModuleId,
    pub name: Ident,
    pub visibility: ItemVisibility,
}

impl NodeInterner {
    pub fn add_reexport(
        &mut self,
        module_def_id: ModuleDefId,
        module_id: ModuleId,
        name: Ident,
        visibility: ItemVisibility,
    ) {
        self.reexports.entry(module_def_id).or_default().push(Reexport {
            module_id,
            name,
            visibility,
        });
    }

    pub fn get_reexports(&self, module_def_id: ModuleDefId) -> &[Reexport] {
        self.reexports.get(&module_def_id).map_or(&[], |reexport| reexport)
    }

    pub fn get_trait_reexports(&self, trait_id: TraitId) -> &[Reexport] {
        self.get_reexports(ModuleDefId::TraitId(trait_id))
    }
}
