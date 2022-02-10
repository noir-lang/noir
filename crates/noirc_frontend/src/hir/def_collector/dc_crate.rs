use super::dc_mod::collect_defs;
use super::errors::DefCollectorErrorKind;
use crate::graph::CrateId;
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleId};
use crate::hir::resolution::resolver::Resolver;
use crate::hir::resolution::{
    import::{resolve_imports, ImportDirective},
    path_resolver::StandardPathResolver,
};
use crate::hir::type_check::type_check_func;
use crate::hir::Context;
use crate::node_interner::{FuncId, NodeInterner, TypeId};
use crate::util::vecmap;
use crate::{NoirFunction, ParsedModule, Path, StructType};
use fm::FileId;
use noirc_errors::CollectedErrors;
use noirc_errors::DiagnosableError;
use std::collections::HashMap;

/// Stores all of the unresolved functions in a particular file/mod
pub struct UnresolvedFunctions {
    pub file_id: FileId,
    pub functions: Vec<(LocalModuleId, FuncId, NoirFunction)>,
}

impl UnresolvedFunctions {
    pub fn push_fn(&mut self, mod_id: LocalModuleId, func_id: FuncId, func: NoirFunction) {
        self.functions.push((mod_id, func_id, func))
    }
}

/// Given a Crate root, collect all definitions in that crate
pub struct DefCollector {
    pub(crate) def_map: CrateDefMap,
    pub(crate) collected_imports: Vec<ImportDirective>,
    pub(crate) collected_functions: Vec<UnresolvedFunctions>,
    pub(crate) collected_types: HashMap<TypeId, StructType>,

    /// collected impls maps the type name and the module id in which
    /// the impl is defined to the functions contained in that impl
    pub(crate) collected_impls: HashMap<(Path, LocalModuleId), Vec<UnresolvedFunctions>>,
}

impl DefCollector {
    fn new(def_map: CrateDefMap) -> DefCollector {
        DefCollector {
            def_map,
            collected_imports: vec![],
            collected_functions: vec![],
            collected_types: HashMap::new(),
            collected_impls: HashMap::new(),
        }
    }

    /// Collect all of the definitions in a given crate into a CrateDefMap
    /// Modules which are not a part of the module hierarchy starting with
    /// the root module, will be ignored.
    pub fn collect(
        mut def_map: CrateDefMap,
        context: &mut Context,
        ast: ParsedModule,
        root_file_id: FileId,
        errors: &mut Vec<CollectedErrors>,
    ) {
        let crate_id = def_map.krate;

        // Recursively resolve the dependencies
        //
        // Dependencies are fetched from the crate graph
        // Then added these to the context of DefMaps once they are resolved
        //
        let crate_graph = &context.crate_graph[crate_id];

        for dep in crate_graph.dependencies.clone() {
            CrateDefMap::collect_defs(dep.crate_id, context, errors);

            let dep_def_root = context
                .def_map(dep.crate_id)
                .expect("ice: def map was just created")
                .root;
            let module_id = ModuleId {
                krate: dep.crate_id,
                local_id: dep_def_root,
            };
            // Add this crate as a dependency by linking it's root module
            def_map.extern_prelude.insert(dep.as_name(), module_id);
        }

        // At this point, all dependencies are resolved and type checked.
        //
        // It is now possible to collect all of the definitions of this crate.
        let crate_root = def_map.root;
        let mut def_collector = DefCollector::new(def_map);

        // Collecting module declarations with ModCollector
        // and lowering the functions
        // i.e. Use a mod collector to collect the nodes at the root module
        // and process them
        collect_defs(
            &mut def_collector,
            ast,
            root_file_id,
            crate_root,
            context,
            errors,
        );

        // Add the current crate to the collection of DefMaps
        context.def_maps.insert(crate_id, def_collector.def_map);

        // Resolve unresolved imports collected from the crate
        let (unresolved, resolved) =
            resolve_imports(crate_id, def_collector.collected_imports, &context.def_maps);

        let current_def_map = context.def_maps.get(&crate_id).unwrap();
        for unresolved_import in unresolved.into_iter() {
            // File if that the import was declared
            let file_id = current_def_map.modules[unresolved_import.module_id.0]
                .origin
                .file_id();
            let diagnostic = DefCollectorErrorKind::UnresolvedImport {
                import: unresolved_import,
            }
            .to_diagnostic();
            let err = CollectedErrors {
                file_id,
                errors: vec![diagnostic],
            };
            errors.push(err);
        }

        // Populate module namespaces according to the imports used
        let current_def_map = context.def_maps.get_mut(&crate_id).unwrap();
        for resolved_import in resolved {
            let name = resolved_import.name;
            for ns in resolved_import.resolved_namespace.iter_defs() {
                let result = current_def_map.modules[resolved_import.module_scope.0]
                    .scope
                    .add_item_to_namespace(name.clone(), ns);

                if let Err((first_def, second_def)) = result {
                    let err = DefCollectorErrorKind::DuplicateImport {
                        first_def,
                        second_def,
                    };

                    errors.push(CollectedErrors {
                        file_id: root_file_id,
                        errors: vec![err.to_diagnostic()],
                    });
                }
            }
        }

        // Create the mappings from TypeId -> StructType
        // so that expressions can access the fields of structs
        for (id, typ) in def_collector.collected_types {
            context.def_interner.push_struct(id, typ);
        }

        let file_method_ids = resolve_impls(
            &mut context.def_interner,
            crate_id,
            &context.def_maps,
            def_collector.collected_impls,
            errors,
        );

        // Lower each function in the crate. This is now possible since imports have been resolved
        let file_func_ids = resolve_functions(
            &mut context.def_interner,
            crate_id,
            &context.def_maps,
            def_collector.collected_functions,
            errors,
        );

        // Type check all of the functions in the crate
        type_check_functions(&mut context.def_interner, file_func_ids, errors);
        type_check_functions(&mut context.def_interner, file_method_ids, errors);
    }
}

fn resolve_impls(
    interner: &mut NodeInterner,
    crate_id: CrateId,
    def_maps: &HashMap<CrateId, CrateDefMap>,
    collected_impls: HashMap<(Path, LocalModuleId), Vec<UnresolvedFunctions>>,
    errors: &mut Vec<CollectedErrors>,
) -> Vec<(FileId, FuncId)> {
    let mut file_method_ids = vec![];

    for ((path, module_id), methods) in collected_impls {
        let mut ids = resolve_functions(interner, crate_id, def_maps, methods, errors);

        let path_resolver = StandardPathResolver::new(ModuleId {
            local_id: module_id,
            krate: crate_id,
        });

        let mut resolver = Resolver::new(interner, &path_resolver, def_maps);
        let type_id = resolver.lookup_type(path);
        if type_id != TypeId::dummy_id() {
            for (_, method_id) in &ids {
                let method_name = resolver.function_name(method_id);
                let typ = resolver.get_struct(type_id);
                let mut typ = typ.borrow_mut();

                // TODO: Check for duplicate functions
                typ.methods.insert(method_name, *method_id);
            }
        }

        file_method_ids.append(&mut ids);
    }

    file_method_ids
}

fn resolve_functions(
    interner: &mut NodeInterner,
    crate_id: CrateId,
    def_maps: &HashMap<CrateId, CrateDefMap>,
    collected_functions: Vec<UnresolvedFunctions>,
    errors: &mut Vec<CollectedErrors>,
) -> Vec<(FileId, FuncId)> {
    let mut file_func_ids = Vec::new();

    // Lower each function in the crate. This is now possible since imports have been resolved
    for unresolved_functions in collected_functions {
        let file_id = unresolved_functions.file_id;
        let mut collected_errors = CollectedErrors {
            file_id,
            errors: Vec::new(),
        };

        for (mod_id, func_id, func) in unresolved_functions.functions {
            file_func_ids.push((file_id, func_id));

            let path_resolver = StandardPathResolver::new(ModuleId {
                local_id: mod_id,
                krate: crate_id,
            });

            let resolver = Resolver::new(interner, &path_resolver, def_maps);

            let (hir_func, func_meta, errs) = resolver.resolve_function(func);
            interner.push_fn_meta(func_meta, func_id);
            interner.update_fn(func_id, hir_func);
            collected_errors
                .errors
                .extend(errs.into_iter().map(|err| err.into_diagnostic(interner)));
        }
        if !collected_errors.errors.is_empty() {
            errors.push(collected_errors);
        }
    }

    file_func_ids
}

fn type_check_functions(
    interner: &mut NodeInterner,
    file_func_ids: Vec<(FileId, FuncId)>,
    errors: &mut Vec<CollectedErrors>,
) {
    file_func_ids
        .into_iter()
        .map(|(file_id, func_id)| {
            let errors = vecmap(type_check_func(interner, func_id), |error| {
                error.into_diagnostic(interner)
            });

            CollectedErrors { file_id, errors }
        })
        .filter(|collected| !collected.errors.is_empty())
        .for_each(|error| errors.push(error));
}
