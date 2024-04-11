use iter_extended::vecmap;
use noirc_errors::Location;
use noirc_frontend::{
    graph::CrateId,
    hir::{
        def_map::{LocalModuleId, ModuleId},
        resolution::{path_resolver::StandardPathResolver, resolver::Resolver},
        type_check::type_check_func,
    },
    macros_api::{FileId, HirContext, ModuleDefId, StructId},
    node_interner::{FuncId, TraitId},
    ItemVisibility, LetStatement, NoirFunction, Shared, Signedness, StructType, Type,
};

use super::ast_utils::is_custom_attribute;

pub fn collect_crate_structs(crate_id: &CrateId, context: &HirContext) -> Vec<(String, StructId)> {
    context
        .def_map(crate_id)
        .expect("ICE: Missing crate in def_map")
        .modules()
        .iter()
        .flat_map(|(_, module)| {
            module.type_definitions().filter_map(move |typ| {
                if let ModuleDefId::TypeId(struct_id) = typ {
                    let module_id = struct_id.module_id();
                    let path =
                        context.fully_qualified_struct_path(context.root_crate_id(), struct_id);
                    let path = if path.contains("::") {
                        let prefix = if &module_id.krate == context.root_crate_id() {
                            "crate"
                        } else {
                            "dep"
                        };
                        format!("{}::{}", prefix, path)
                    } else {
                        path
                    };

                    Some((path, struct_id))
                } else {
                    None
                }
            })
        })
        .collect()
}

pub fn collect_crate_functions(crate_id: &CrateId, context: &HirContext) -> Vec<FuncId> {
    context
        .def_map(crate_id)
        .expect("ICE: Missing crate in def_map")
        .modules()
        .iter()
        .flat_map(|(_, module)| module.value_definitions().filter_map(|id| id.as_function()))
        .collect()
}

pub fn collect_traits(context: &HirContext) -> Vec<TraitId> {
    let crates = context.crates();
    crates
        .flat_map(|crate_id| context.def_map(&crate_id).map(|def_map| def_map.modules()))
        .flatten()
        .flat_map(|module| {
            module.type_definitions().filter_map(|typ| {
                if let ModuleDefId::TraitId(trait_id) = typ {
                    Some(trait_id)
                } else {
                    None
                }
            })
        })
        .collect()
}

/// Computes the aztec signature for a resolved type.
pub fn signature_of_type(typ: &Type) -> String {
    match typ {
        Type::Integer(Signedness::Signed, bit_size) => format!("i{}", bit_size),
        Type::Integer(Signedness::Unsigned, bit_size) => format!("u{}", bit_size),
        Type::FieldElement => "Field".to_owned(),
        Type::Bool => "bool".to_owned(),
        Type::Array(len, typ) => {
            if let Type::Constant(len) = **len {
                format!("[{};{len}]", signature_of_type(typ))
            } else {
                unimplemented!("Cannot generate signature for array with length type {:?}", typ)
            }
        }
        Type::Struct(def, args) => {
            let fields = def.borrow().get_fields(args);
            let fields = vecmap(fields, |(_, typ)| signature_of_type(&typ));
            format!("({})", fields.join(","))
        }
        Type::Tuple(types) => {
            let fields = vecmap(types, signature_of_type);
            format!("({})", fields.join(","))
        }
        _ => unimplemented!("Cannot generate signature for type {:?}", typ),
    }
}

// Fetches the name of all structs tagged as #[aztec(note)] in a given crate
pub fn fetch_crate_notes(
    context: &HirContext,
    crate_id: &CrateId,
) -> Vec<(String, Shared<StructType>)> {
    collect_crate_structs(crate_id, context)
        .iter()
        .filter_map(|(path, struct_id)| {
            let r#struct = context.def_interner.get_struct(*struct_id);
            let attributes = context.def_interner.struct_attributes(struct_id);
            if attributes.iter().any(|attr| is_custom_attribute(attr, "aztec(note)")) {
                Some((path.clone(), r#struct))
            } else {
                None
            }
        })
        .collect()
}

// Fetches the name of all structs tagged as #[aztec(note)], both in the current crate and all of its dependencies.
pub fn fetch_notes(context: &HirContext) -> Vec<(String, Shared<StructType>)> {
    context.crates().flat_map(|crate_id| fetch_crate_notes(context, &crate_id)).collect()
}

pub fn get_contract_module_data(
    context: &mut HirContext,
    crate_id: &CrateId,
) -> Option<(LocalModuleId, FileId)> {
    // We first fetch modules in this crate which correspond to contracts, along with their file id.
    let contract_module_file_ids: Vec<(LocalModuleId, FileId)> = context
        .def_map(crate_id)
        .expect("ICE: Missing crate in def_map")
        .modules()
        .iter()
        .filter(|(_, module)| module.is_contract)
        .map(|(idx, module)| (LocalModuleId(idx), module.location.file))
        .collect();

    // If the current crate does not contain a contract module we simply skip it. More than 1 contract in a crate is forbidden by the compiler
    if contract_module_file_ids.is_empty() {
        return None;
    }

    Some(contract_module_file_ids[0])
}

pub fn inject_fn(
    crate_id: &CrateId,
    context: &mut HirContext,
    func: NoirFunction,
    location: Location,
    module_id: LocalModuleId,
    file_id: FileId,
) {
    let func_id = context.def_interner.push_empty_fn();
    context.def_interner.push_function(
        func_id,
        &func.def,
        ModuleId { krate: *crate_id, local_id: module_id },
        location,
    );

    context.def_map_mut(crate_id).unwrap().modules_mut()[module_id.0]
        .declare_function(func.name_ident().clone(), ItemVisibility::Public, func_id)
        .unwrap_or_else(|_| {
            panic!(
                "Failed to declare autogenerated {} function, likely due to a duplicate definition",
                func.name()
            )
        });

    let def_maps = &mut context.def_maps;

    let path_resolver =
        StandardPathResolver::new(ModuleId { local_id: module_id, krate: *crate_id });

    let resolver = Resolver::new(&mut context.def_interner, &path_resolver, def_maps, file_id);

    let (hir_func, meta, _) = resolver.resolve_function(func, func_id);

    context.def_interner.push_fn_meta(meta, func_id);
    context.def_interner.update_fn(func_id, hir_func);

    type_check_func(&mut context.def_interner, func_id);
}

pub fn inject_global(
    crate_id: &CrateId,
    context: &mut HirContext,
    global: LetStatement,
    module_id: LocalModuleId,
    file_id: FileId,
) {
    let name = global.pattern.name_ident().clone();

    let global_id = context.def_interner.push_empty_global(
        name.clone(),
        module_id,
        file_id,
        global.attributes.clone(),
    );

    // Add the statement to the scope so its path can be looked up later
    context.def_map_mut(crate_id).unwrap().modules_mut()[module_id.0]
        .declare_global(name, global_id)
        .unwrap_or_else(|(name, _)| {
            panic!(
                "Failed to declare autogenerated {} global, likely due to a duplicate definition",
                name
            )
        });

    let def_maps = &mut context.def_maps;

    let path_resolver =
        StandardPathResolver::new(ModuleId { local_id: module_id, krate: *crate_id });

    let mut resolver = Resolver::new(&mut context.def_interner, &path_resolver, def_maps, file_id);

    let hir_stmt = resolver.resolve_global_let(global, global_id);

    let statement_id = context.def_interner.get_global(global_id).let_statement;
    context.def_interner.replace_statement(statement_id, hir_stmt);
}
