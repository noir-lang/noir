use noirc_frontend::{ast::ItemVisibility, hir::def_map::ModuleId};

pub(super) fn is_visible(
    visibility: ItemVisibility,
    current_module: ModuleId,
    target_module: ModuleId,
) -> bool {
    match visibility {
        ItemVisibility::Public => true,
        ItemVisibility::Private => false,
        ItemVisibility::PublicCrate => current_module.krate == target_module.krate,
    }
}
