use super::dc_mod::ModCollector;
use crate::graph::CrateId;
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleId};
use crate::hir::resolution::resolver::Resolver;
use crate::hir::resolution::{
    import::{resolve_imports, ImportDirective},
    path_resolver::FunctionPathResolver,
};
use crate::hir::Context;
use crate::node_interner::{FuncId, NodeInterner};
use crate::{NoirFunction, Program};
use fm::FileId;
use noirc_errors::CollectedErrors;
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
}

impl DefCollector {
    /// Collect all of the definitions in a given crate into a CrateDefMap
    /// Modules which are not a part of the module hierarchy starting with
    /// the root module, will be ignored.
    pub fn collect(
        mut def_map: CrateDefMap,
        context: &mut Context,
        ast: Program,
        root_file_id: FileId,
    ) -> Result<(), Vec<CollectedErrors>> {
        let crate_id = def_map.krate;

        // Recursively resolve the dependencies
        //
        // Dependencies are fetched from the crate graph
        // Then added these to the context of DefMaps once they are resolved
        //
        let crate_graph = &context.crate_graph()[crate_id];
        for dep in crate_graph.dependencies.clone() {
            CrateDefMap::collect_defs(dep.crate_id, context)?;

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
        let mut def_collector = DefCollector {
            def_map,
            collected_imports: Vec::new(),
            collected_functions: Vec::new(),
        };

        // Collecting module declarations with ModCollector
        // and lowering the functions
        // ie Use a mod collector to collect the nodes at the root module
        // and process them
        ModCollector {
            def_collector: &mut def_collector,
            ast,
            file_id: root_file_id,
            module_id: crate_root,
        }
        .collect_defs(context)?;

        // Add the current crate to the collection of DefMaps
        context.def_maps.insert(crate_id, def_collector.def_map);

        // Resolve unresolved imports collected from the crate
        let (unresolved, resolved) =
            resolve_imports(crate_id, def_collector.collected_imports, &context.def_maps);
        if !unresolved.is_empty() {
            panic!("could not resolve the following imports: {:?}", unresolved)
        }

        // Populate module namespaces according to the imports used
        let current_def_map = context.def_maps.get_mut(&crate_id).unwrap();
        for resolved_import in resolved {
            let name = resolved_import.name;
            for ns in resolved_import.resolved_namespace.iter_defs() {
                current_def_map.modules[resolved_import.module_scope.0]
                    .scope
                    .add_item_to_namespace(name.clone(), ns)
                    .expect("could not add item to namespace");
            }
        }

        // Lower each function in the crate. This is now possible since imports have been resolved
        let file_func_ids = resolve_functions(
            &mut context.def_interner,
            crate_id,
            &context.def_maps,
            def_collector.collected_functions,
        )?;

        // Type check all of the functions in the crate
        type_check_functions(&mut context.def_interner, file_func_ids)?;

        Ok(())
    }
}

fn resolve_functions(
    interner: &mut NodeInterner,
    crate_id: CrateId,
    def_maps: &HashMap<CrateId, CrateDefMap>,
    collected_functions: Vec<UnresolvedFunctions>,
) -> Result<Vec<(FileId, FuncId)>, Vec<CollectedErrors>> {
    let mut file_func_ids = Vec::new();
    let mut errors = Vec::new();

    // Lower each function in the crate. This is now possible since imports have been resolved
    for unresolved_functions in collected_functions {
        let file_id = unresolved_functions.file_id;
        let mut collected_errors = CollectedErrors {
            file_id,
            errors: Vec::new(),
        };

        for (mod_id, func_id, func) in unresolved_functions.functions {
            file_func_ids.push((file_id, func_id));

            let func_resolver = FunctionPathResolver::new(ModuleId {
                local_id: mod_id,
                krate: crate_id,
            });
            let resolver = Resolver::new(interner, &func_resolver, def_maps);

            match resolver.resolve_function(func) {
                Ok((hir_func, func_meta)) => {
                    interner.push_fn_meta(func_meta, func_id);
                    interner.update_fn(func_id, hir_func);
                }
                Err(errs) => {
                    collected_errors
                        .errors
                        .extend(errs.into_iter().map(|err| err.into_diagnostic(&interner)));
                }
            }
        }
        if !collected_errors.errors.is_empty() {
            errors.push(collected_errors);
        }
    }

    if errors.is_empty() {
        return Ok(file_func_ids);
    }
    return Err(errors);
}

use crate::hir::type_check::type_check_func;
fn type_check_functions(
    interner: &mut NodeInterner,
    file_func_ids: Vec<(FileId, FuncId)>,
) -> Result<(), Vec<CollectedErrors>> {
    for (file_id, func_id) in file_func_ids {
        if let Err(type_err) = type_check_func(interner, func_id) {
            let diag = type_err.into_diagnostics(interner);
            let errs = vec![CollectedErrors {
                file_id,
                errors: diag,
            }];
            return Err(errs);
        }
    }

    Ok(())
}
