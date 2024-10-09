use std::collections::BTreeMap;

use noirc_frontend::{
    ast::ItemVisibility,
    graph::CrateId,
    hir::{
        def_map::{CrateDefMap, ModuleId},
        resolution::visibility::can_reference_module_id,
    },
};

pub(super) fn is_visible(
    target_module_id: ModuleId,
    current_module_id: ModuleId,
    visibility: ItemVisibility,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
) -> bool {
    can_reference_module_id(
        def_maps,
        current_module_id.krate,
        current_module_id.local_id,
        target_module_id,
        visibility,
    )
}
