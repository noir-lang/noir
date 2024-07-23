use std::collections::BTreeMap;

use fm::FileId;

use crate::ast::ItemVisibility;
use crate::{
    graph::CrateId,
    hir::{
        def_collector::{
            dc_crate::{CompilationError, ImplMap},
            errors::DefCollectorErrorKind,
        },
        def_map::{CrateDefMap, ModuleId},
        Context,
    },
    node_interner::{FuncId, NodeInterner},
    Type,
};

use super::{
    errors::ResolverError, functions, get_module_mut, get_struct_type,
    path_resolver::StandardPathResolver, resolver::Resolver, take_errors,
};

/// Go through the list of impls and add each function within to the scope
/// of the module defined by its type.
pub(crate) fn collect_impls(
    context: &mut Context,
    crate_id: CrateId,
    collected_impls: &ImplMap,
) -> Vec<(CompilationError, FileId)> {
    let interner = &mut context.def_interner;
    let def_maps = &mut context.def_maps;
    let mut errors: Vec<(CompilationError, FileId)> = vec![];

    for ((unresolved_type, module_id), methods) in collected_impls {
        let path_resolver =
            StandardPathResolver::new(ModuleId { local_id: *module_id, krate: crate_id });

        let file = def_maps[&crate_id].file_id(*module_id);

        for (generics, span, unresolved) in methods {
            let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file);
            resolver.add_generics(generics);
            let typ = resolver.resolve_type(unresolved_type.clone());

            errors.extend(take_errors(unresolved.file_id, resolver));

            if let Some(struct_type) = get_struct_type(&typ) {
                let struct_type = struct_type.borrow();

                // `impl`s are only allowed on types defined within the current crate
                if struct_type.id.krate() != crate_id {
                    let span = *span;
                    let type_name = struct_type.name.to_string();
                    let error = DefCollectorErrorKind::ForeignImpl { span, type_name };
                    errors.push((error.into(), unresolved.file_id));
                    continue;
                }

                // Grab the module defined by the struct type. Note that impls are a case
                // where the module the methods are added to is not the same as the module
                // they are resolved in.
                let module = get_module_mut(def_maps, struct_type.id.module_id());

                for (_, method_id, method) in &unresolved.functions {
                    // If this method was already declared, remove it from the module so it cannot
                    // be accessed with the `TypeName::method` syntax. We'll check later whether the
                    // object types in each method overlap or not. If they do, we issue an error.
                    // If not, that is specialization which is allowed.
                    if module
                        .declare_function(
                            method.name_ident().clone(),
                            ItemVisibility::Public,
                            *method_id,
                        )
                        .is_err()
                    {
                        module.remove_function(method.name_ident());
                    }
                }
            // Prohibit defining impls for primitive types if we're not in the stdlib
            } else if typ != Type::Error && !crate_id.is_stdlib() {
                let span = *span;
                let error = DefCollectorErrorKind::NonStructTypeInImpl { span };
                errors.push((error.into(), unresolved.file_id));
            }
        }
    }
    errors
}

pub(crate) fn resolve_impls(
    interner: &mut NodeInterner,
    crate_id: CrateId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    collected_impls: ImplMap,
    errors: &mut Vec<(CompilationError, FileId)>,
) -> Vec<(FileId, FuncId)> {
    let mut file_method_ids = Vec::new();

    for ((unresolved_type, module_id), methods) in collected_impls {
        let path_resolver =
            StandardPathResolver::new(ModuleId { local_id: module_id, krate: crate_id });

        let file = def_maps[&crate_id].file_id(module_id);

        for (generics, _, functions) in methods {
            let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file);
            resolver.add_generics(&generics);
            let generics = resolver.get_generics().to_vec();
            let self_type = resolver.resolve_type(unresolved_type.clone());

            let mut file_func_ids = functions::resolve_function_set(
                interner,
                crate_id,
                def_maps,
                functions,
                Some(self_type.clone()),
                None,
                generics,
                errors,
            );
            if self_type != Type::Error {
                for (file_id, method_id) in &file_func_ids {
                    let method_name = interner.function_name(method_id).to_owned();

                    if let Some(first_fn) =
                        interner.add_method(&self_type, method_name.clone(), *method_id, false)
                    {
                        let error = ResolverError::DuplicateDefinition {
                            name: method_name,
                            first_span: interner.function_ident(&first_fn).span(),
                            second_span: interner.function_ident(method_id).span(),
                        };
                        errors.push((error.into(), *file_id));
                    }
                }
            }
            file_method_ids.append(&mut file_func_ids);
        }
    }

    file_method_ids
}
