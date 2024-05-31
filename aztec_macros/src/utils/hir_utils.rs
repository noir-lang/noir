use acvm::acir::AcirField;
use iter_extended::vecmap;
use noirc_errors::Location;
use noirc_frontend::ast;
use noirc_frontend::macros_api::{HirExpression, HirLiteral};
use noirc_frontend::node_interner::{NodeInterner, TraitImplKind};
use noirc_frontend::{
    graph::CrateId,
    hir::{
        def_map::{LocalModuleId, ModuleId},
        resolution::{path_resolver::StandardPathResolver, resolver::Resolver},
        type_check::type_check_func,
    },
    macros_api::{FileId, HirContext, MacroError, ModuleDefId, StructId},
    node_interner::{FuncId, TraitId},
    Shared, StructType, Type,
};

use super::ast_utils::is_custom_attribute;

pub fn collect_crate_structs(crate_id: &CrateId, context: &HirContext) -> Vec<StructId> {
    context
        .def_map(crate_id)
        .map(|def_map| {
            def_map
                .modules()
                .iter()
                .flat_map(|(_, module)| {
                    module.type_definitions().filter_map(move |typ| {
                        if let ModuleDefId::TypeId(struct_id) = typ {
                            Some(struct_id)
                        } else {
                            None
                        }
                    })
                })
                .collect()
        })
        .unwrap_or_default()
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
        Type::Integer(ast::Signedness::Signed, bit_size) => format!("i{}", bit_size),
        Type::Integer(ast::Signedness::Unsigned, bit_size) => format!("u{}", bit_size),
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
        Type::String(len_typ) => {
            if let Type::Constant(len) = **len_typ {
                format!("str<{len}>")
            } else {
                unimplemented!(
                    "Cannot generate signature for string with length type {:?}",
                    len_typ
                )
            }
        }
        Type::MutableReference(typ) => signature_of_type(typ),
        _ => unimplemented!("Cannot generate signature for type {:?}", typ),
    }
}

// Fetches the name of all structs tagged as #[aztec(note)] in a given crate, avoiding
// contract dependencies that are just there for their interfaces.
pub fn fetch_crate_notes(
    context: &HirContext,
    crate_id: &CrateId,
) -> Vec<(String, Shared<StructType>)> {
    collect_crate_structs(crate_id, context)
        .iter()
        .filter_map(|struct_id| {
            let r#struct = context.def_interner.get_struct(*struct_id);
            let attributes = context.def_interner.struct_attributes(struct_id);
            if attributes.iter().any(|attr| is_custom_attribute(attr, "aztec(note)")) {
                let module_id = struct_id.module_id();

                fully_qualified_note_path(context, *struct_id).map(|path| {
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
                    (path.clone(), r#struct)
                })
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
) -> Option<(String, LocalModuleId, FileId)> {
    let def_map = context.def_map(crate_id).expect("ICE: Missing crate in def_map");
    // We first fetch modules in this crate which correspond to contracts, along with their file id.
    let contract_module_file_ids: Vec<(String, LocalModuleId, FileId)> = def_map
        .modules()
        .iter()
        .filter(|(_, module)| module.is_contract)
        .map(|(idx, module)| {
            (def_map.get_module_path(idx, module.parent), LocalModuleId(idx), module.location.file)
        })
        .collect();

    // If the current crate does not contain a contract module we simply skip it.
    if contract_module_file_ids.is_empty() {
        return None;
    }

    Some(contract_module_file_ids[0].clone())
}

pub fn inject_fn(
    crate_id: &CrateId,
    context: &mut HirContext,
    func: ast::NoirFunction,
    location: Location,
    module_id: LocalModuleId,
    file_id: FileId,
) -> Result<(), MacroError> {
    let func_id = context.def_interner.push_empty_fn();
    context.def_interner.push_function(
        func_id,
        &func.def,
        ModuleId { krate: *crate_id, local_id: module_id },
        location,
    );

    context.def_map_mut(crate_id).unwrap().modules_mut()[module_id.0]
        .declare_function(func.name_ident().clone(), ast::ItemVisibility::Public, func_id)
        .map_err(|err| MacroError {
            primary_message: format!("Failed to declare autogenerated {} function", func.name()),
            secondary_message: Some(format!("Duplicate definition found {}", err.0)),
            span: None,
        })?;

    let def_maps = &mut context.def_maps;

    let path_resolver =
        StandardPathResolver::new(ModuleId { local_id: module_id, krate: *crate_id });

    let resolver = Resolver::new(&mut context.def_interner, &path_resolver, def_maps, file_id);

    let (hir_func, meta, _) = resolver.resolve_function(func, func_id);

    context.def_interner.push_fn_meta(meta, func_id);
    context.def_interner.update_fn(func_id, hir_func);

    let errors = type_check_func(&mut context.def_interner, func_id);

    if !errors.is_empty() {
        return Err(MacroError {
            primary_message: "Failed to type check autogenerated function".to_owned(),
            secondary_message: Some(errors.iter().map(|err| err.to_string()).collect::<String>()),
            span: None,
        });
    }

    Ok(())
}

pub fn inject_global(
    crate_id: &CrateId,
    context: &mut HirContext,
    global: ast::LetStatement,
    module_id: LocalModuleId,
    file_id: FileId,
) {
    let name = global.pattern.name_ident().clone();

    let global_id = context.def_interner.push_empty_global(
        name.clone(),
        module_id,
        file_id,
        global.attributes.clone(),
        false,
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

pub fn fully_qualified_note_path(context: &HirContext, note_id: StructId) -> Option<String> {
    let module_id = note_id.module_id();
    let child_id = module_id.local_id.0;
    let def_map =
        context.def_map(&module_id.krate).expect("The local crate should be analyzed already");

    let module = context.module(module_id);

    let module_path = def_map.get_module_path_with_separator(child_id, module.parent, "::");

    if &module_id.krate == context.root_crate_id() {
        Some(module_path)
    } else {
        find_non_contract_dependencies_bfs(context, context.root_crate_id(), &module_id.krate)
            .map(|crates| crates.join("::") + "::" + &module_path)
    }
}

fn filter_contract_modules(context: &HirContext, crate_id: &CrateId) -> bool {
    if let Some(def_map) = context.def_map(crate_id) {
        !def_map.modules().iter().any(|(_, module)| module.is_contract)
    } else {
        true
    }
}

fn find_non_contract_dependencies_bfs(
    context: &HirContext,
    crate_id: &CrateId,
    target_crate_id: &CrateId,
) -> Option<Vec<String>> {
    context.crate_graph[crate_id]
        .dependencies
        .iter()
        .filter(|dep| filter_contract_modules(context, &dep.crate_id))
        .find_map(|dep| {
            if &dep.crate_id == target_crate_id {
                Some(vec![dep.name.to_string()])
            } else {
                None
            }
        })
        .or_else(|| {
            context.crate_graph[crate_id]
                .dependencies
                .iter()
                .filter(|dep| filter_contract_modules(context, &dep.crate_id))
                .find_map(|dep| {
                    if let Some(mut path) =
                        find_non_contract_dependencies_bfs(context, &dep.crate_id, target_crate_id)
                    {
                        path.insert(0, dep.name.to_string());
                        Some(path)
                    } else {
                        None
                    }
                })
        })
}

pub fn get_serialized_length(
    traits: &[TraitId],
    trait_name: &str,
    typ: &Type,
    interner: &NodeInterner,
) -> Result<u64, MacroError> {
    let serialized_trait_impl_kind = traits
        .iter()
        .find_map(|&trait_id| {
            let r#trait = interner.get_trait(trait_id);
            if r#trait.name.0.contents == trait_name {
                interner.lookup_all_trait_implementations(typ, trait_id).into_iter().next()
            } else {
                None
            }
        })
        .ok_or(MacroError {
            primary_message: format!("Type {} must implement {} trait", typ, trait_name),
            secondary_message: None,
            span: None,
        })?;

    let serialized_trait_impl_id = match serialized_trait_impl_kind {
        TraitImplKind::Normal(trait_impl_id) => Ok(trait_impl_id),
        _ => Err(MacroError {
            primary_message: format!("{} trait impl for {} must not be assumed", trait_name, typ),
            secondary_message: None,
            span: None,
        }),
    }?;

    let serialized_trait_impl_shared = interner.get_trait_implementation(*serialized_trait_impl_id);
    let serialized_trait_impl = serialized_trait_impl_shared.borrow();

    match serialized_trait_impl.trait_generics.first().unwrap() {
        Type::Constant(value) => Ok(*value),
        _ => Err(MacroError {
            primary_message: format!("{} length for {} must be a constant", trait_name, typ),
            secondary_message: None,
            span: None,
        }),
    }
}

pub fn get_global_numberic_const(
    context: &HirContext,
    const_name: &str,
) -> Result<u128, MacroError> {
    context
        .def_interner
        .get_all_globals()
        .iter()
        .find_map(|global_info| {
            if global_info.ident.0.contents == const_name {
                let stmt = context.def_interner.get_global_let_statement(global_info.id);
                if let Some(let_stmt) = stmt {
                    let expression = context.def_interner.expression(&let_stmt.expression);
                    match expression {
                        HirExpression::Literal(HirLiteral::Integer(value, _)) => {
                            Some(value.to_u128())
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .ok_or(MacroError {
            primary_message: format!("Could not find {} global constant", const_name),
            secondary_message: None,
            span: None,
        })
}
