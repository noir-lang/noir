use nargo_doc::links::CurrentType;
use noirc_frontend::hir::def_map::ModuleDefId;
use noirc_frontend::modules::get_parent_module;
use noirc_frontend::{Type, hir::def_map::ModuleId, node_interner::ReferenceId};

use crate::requests::ProcessRequestCallbackArgs;

/// Given a ReferenceId, returns the ModuleId it belongs to along with whether it represents
/// a `CurrentType`.
pub(crate) fn current_module_and_type(
    id: ReferenceId,
    args: &ProcessRequestCallbackArgs,
) -> Option<(ModuleId, Option<CurrentType>)> {
    match id {
        ReferenceId::Module(module_id) => {
            let parent_module =
                get_parent_module(ModuleDefId::ModuleId(module_id), args.interner, args.def_maps);
            if let Some(parent_module) = parent_module {
                Some((parent_module, None))
            } else {
                // If there's no parent module, it means we are in the crate root module
                Some((module_id, None))
            }
        }
        ReferenceId::Type(type_id)
        | ReferenceId::StructMember(type_id, _)
        | ReferenceId::EnumVariant(type_id, _) => {
            let parent_module =
                get_parent_module(ModuleDefId::TypeId(type_id), args.interner, args.def_maps)?;
            Some((parent_module, Some(CurrentType::Type(type_id))))
        }
        ReferenceId::Trait(trait_id) => {
            let parent_module =
                get_parent_module(ModuleDefId::TraitId(trait_id), args.interner, args.def_maps)?;
            Some((parent_module, Some(CurrentType::Trait(trait_id))))
        }
        ReferenceId::TraitAssociatedType(trait_associated_type_id) => {
            let associated_type = args.interner.get_trait_associated_type(trait_associated_type_id);
            let trait_id = associated_type.trait_id;
            let parent_module =
                get_parent_module(ModuleDefId::TraitId(trait_id), args.interner, args.def_maps)?;
            Some((parent_module, Some(CurrentType::Trait(trait_id))))
        }
        ReferenceId::Global(global_id) => {
            let parent_module =
                get_parent_module(ModuleDefId::GlobalId(global_id), args.interner, args.def_maps)?;
            Some((parent_module, None))
        }
        ReferenceId::Function(func_id) => {
            let func_meta = args.interner.function_meta(&func_id);
            let current_type = match &func_meta.self_type {
                Some(Type::DataType(data_type, _)) => {
                    Some(CurrentType::Type(data_type.borrow().id))
                }
                _ => func_meta.trait_id.map(CurrentType::Trait),
            };
            let parent_module =
                get_parent_module(ModuleDefId::FunctionId(func_id), args.interner, args.def_maps)?;
            Some((parent_module, current_type))
        }
        ReferenceId::Alias(type_alias_id) => {
            let parent_module = get_parent_module(
                ModuleDefId::TypeAliasId(type_alias_id),
                args.interner,
                args.def_maps,
            )?;
            Some((parent_module, None))
        }
        ReferenceId::Local(..) | ReferenceId::Reference(..) => None,
    }
}
