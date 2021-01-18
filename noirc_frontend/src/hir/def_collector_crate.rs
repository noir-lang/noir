
use fm::FileId;

use crate::{NoirFunction, Program};

use super::{Context, crate_def_map::{CrateDefMap, LocalModuleId, ModuleId, ModuleOrigin}, def_collector_mod::ModCollector, lower::{def_interner::FuncId, resolver::Resolver}, resolution::{FunctionPathResolver, import::ImportDirective}, type_check::TypeChecker};

/// Given a Crate root, collect all definitions in that crate
pub struct DefCollector {
    pub(crate) def_map : CrateDefMap,
    pub(crate) collected_imports : Vec<ImportDirective>,
    pub(crate) collected_functions : Vec<(LocalModuleId, FuncId, NoirFunction)>,
}

impl DefCollector {
    /// Collect all of the definitions in a given crate into a CrateDefMap
    /// Modules which are not a part of the module hierarchy will be ignored.
    pub fn collect(mut def_map : CrateDefMap, mut context : &mut Context ,ast : Program, root_file_id : FileId) {

        let crate_id = def_map.krate;

        // First collect all of the definitions from the crate dependencies into CrateDefMaps
        // Dependencies are fetched from the crate graph
        // Then add these to the context of DefMaps
        let crate_graph = &context.crate_graph()[crate_id];
        for dep in crate_graph.dependencies.clone() {
            CrateDefMap::collect_defs(dep.crate_id, &mut context);
            let dep_def_root = context.def_map(dep.crate_id).expect("ice: def map was just created").root;
            def_map.extern_prelude.insert(dep.as_name(), ModuleId{krate : dep.crate_id, local_id : dep_def_root});
        }
        // Get the module associated with the root of the crate
        // Since Macros are not being used (like Rust), this will have a one to one mapping 
        // to file Id
        let module_id = def_map.root;

        // Populate the Preallocated ModuleId to be the origin
        // Note this rests on the fact that the root file already has a module allocated
        def_map[module_id].origin = ModuleOrigin::CrateRoot(root_file_id);

        let mut def_collector = DefCollector {
            def_map,
            collected_imports : Vec::new(),
            collected_functions : Vec::new(),
        };


        // Resolving module declarations with ModCollector
        // and lowering the functions
        // ie Use a mod collector to collect the nodes at the root module
        // and process them
        ModCollector {
            def_collector : &mut def_collector,
            ast,
            file_id : root_file_id,
            module_id : module_id,
        }.collect_defs(context);

        // Add the current crate to the collection of DefMaps
        let old_value = context.def_maps.insert(crate_id, def_collector.def_map);
        assert!(old_value.is_none(), "value : {:?}", old_value);

        // Resolve unresolved imports collected from the crate
        let (unresolved, resolved) = super::resolution::import::resolve_imports(crate_id, def_collector.collected_imports, &context.def_maps);
        if !unresolved.is_empty() {
            panic!(format!("could not resolve the following imports: {:?}", unresolved))
        }

        // Populate module namespaces according to the imports used
        let current_def_map = context.def_maps.get_mut(&crate_id).unwrap();
        for resolved_import in resolved {
            let name = resolved_import.name;
            for ns in resolved_import.resolved_namespace.iter_defs() {
                current_def_map.modules[resolved_import.module_scope.0].scope.add_item_to_namespace(name.clone(), ns).expect("could not add item to namespace");
            }
        }
        
        let func_ids : Vec<_> = def_collector.collected_functions.iter().map(|(_,f_id, _)|f_id).copied().collect();

        // Lower each function in the crate. This is now possible since imports have been resolved
        for (mod_id, func_id, func) in def_collector.collected_functions {
            let func_resolver = FunctionPathResolver::new(ModuleId{local_id : mod_id, krate: crate_id});
            let mut resolver = Resolver::new(&mut context.def_interner, &func_resolver, &context.def_maps);
            
            let (hir_func, func_meta) = resolver.resolve_function(func);
            
            context.def_interner.push_fn_meta(func_meta, func_id);
            context.def_interner.update_fn(func_id, hir_func);
        } 

       // Type check all of the functions in the crate
       let mut type_checker = TypeChecker::new(&mut context.def_interner);
       for func_id in func_ids {
            type_checker.check_func(func_id);
       }
       
    }
}