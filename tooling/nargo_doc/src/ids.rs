use noirc_frontend::hir::def_map::{ModuleDefId, ModuleId};
use noirc_frontend::node_interner::NodeInterner;
use noirc_frontend::node_interner::{FuncId, GlobalId, TraitId, TypeAliasId, TypeId};

use crate::items::{ItemId, ItemKind};

pub(crate) fn get_module_def_id(id: ModuleDefId, interner: &NodeInterner) -> ItemId {
    match id {
        ModuleDefId::ModuleId(id) => get_module_id(id, interner),
        ModuleDefId::FunctionId(id) => get_function_id(id, interner),
        ModuleDefId::TypeId(id) => get_type_id(id, interner),
        ModuleDefId::TypeAliasId(id) => get_type_alias_id(id, interner),
        ModuleDefId::TraitId(id) => get_trait_id(id, interner),
        ModuleDefId::GlobalId(id) => get_global_id(id, interner),
        ModuleDefId::TraitAssociatedTypeId(_) => {
            panic!("Trait associated types cannot be re-exported")
        }
    }
}

pub(crate) fn get_module_id(id: ModuleId, interner: &NodeInterner) -> ItemId {
    let module = interner.module_attributes(id);
    let location = module.location;
    let name = module.name.clone();
    let kind = ItemKind::Module;
    ItemId { location, kind, name }
}

pub(crate) fn get_type_id(id: TypeId, interner: &NodeInterner) -> ItemId {
    let data_type = interner.get_type(id);
    let data_type = data_type.borrow();
    let location = data_type.location;
    let name = data_type.name.to_string();
    let kind = ItemKind::Struct;
    ItemId { location, kind, name }
}

pub(crate) fn get_trait_id(id: TraitId, interner: &NodeInterner) -> ItemId {
    let trait_ = interner.get_trait(id);
    let location = trait_.location;
    let name = trait_.name.to_string();
    let kind = ItemKind::Trait;
    ItemId { location, kind, name }
}

pub(crate) fn get_type_alias_id(id: TypeAliasId, interner: &NodeInterner) -> ItemId {
    let alias = interner.get_type_alias(id);
    let alias = alias.borrow();
    let location = alias.location;
    let name = alias.name.to_string();
    let kind = ItemKind::TypeAlias;
    ItemId { location, kind, name }
}

pub(crate) fn get_function_id(id: FuncId, interner: &NodeInterner) -> ItemId {
    let func_meta = interner.function_meta(&id);
    let name = interner.function_name(&id).to_owned();
    let location = func_meta.location;
    let kind = ItemKind::Function;
    ItemId { location, kind, name }
}

pub(crate) fn get_global_id(id: GlobalId, interner: &NodeInterner) -> ItemId {
    let global_info = interner.get_global(id);
    let location = global_info.location;
    let name = global_info.ident.to_string();
    let kind = ItemKind::Global;
    ItemId { location, kind, name }
}
