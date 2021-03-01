use std::path::PathBuf;

use fm::FileManager;
use hir::{
    crate_def_map::CrateDefMap,
    crate_graph::{CrateGraph, CrateType},
    Context,
};
use noirc_frontend::hir::{self, crate_def_map::ModuleDefId};

// XXX: This is another sandbox test
fn main() {
    // File Manager
    let mut fm = FileManager::new();
    //
    // Add root file to file manager
    let dir_path: PathBuf = PathBuf::from("example_project/lib.nr");
    let root_file_id = fm.add_file(&dir_path).unwrap();

    // CrateGraph
    let mut crate_graph = CrateGraph::default();
    // Initiate crate with root file
    let crate_id = crate_graph.add_crate_root(CrateType::Library, root_file_id);

    // initiate context with file manager and crate graph
    let mut context = Context::new(fm, crate_graph);

    // Now create the CrateDefMap
    // This is preamble for analysis
    // With a CrateDefMap, we are sure that the imports are correct, and the modules declared are located
    // The modules are resolved and type checked!
    CrateDefMap::collect_defs(crate_id, &mut context).unwrap();
    let def_map = context.def_map(crate_id).unwrap();

    //

    // Get root module
    let root = def_map.root();
    let module = def_map.modules().get(root.0).unwrap();
    for (name, (def_id, vis)) in module.scope.values() {
        println!("func name is {}", name);
        let func_id = match def_id {
            ModuleDefId::FunctionId(func_id) => func_id,
            _ => unreachable!(),
        };

        // Get the HirFunction for that Id
        let hir = context.def_interner.function(func_id);

        println!("func hir is {:?}", hir);
        println!("func vis is {:?}", vis);
    }
    //

    // println!("----------------------------CRATE DATA------------------------------");
    // println!("Local module id is: {:?}", def_map.root());
    // println!("crate id is: {:?}", def_map.krate());
    // println!("-----------------------------MODULES DATA----------------------------");
    // for (i, (module_index, module_data)) in def_map.modules().iter().enumerate() {
    //     println!("-----------------------------Start Data for module at position {}----------------------------", i);
    //     println!("current module id is: {:?}", module_index);
    //     println!("parent module id is: {:?}", module_data.parent);
    //     println!("-----------------------------Chidren for module at position {}----------------------------", i);
    //     for (child_name, child_id) in &module_data.children {
    //         println!("{:?} is a child module with id {:?}", child_name, child_id);
    //     }
    //     println!("scope is : {:?}", module_data.scope);
    //     println!("origin is : {:?}", module_data.origin);
    //     println!("-----------------------------End Data for module at position {}----------------------------", i);
    // }
    // println!("-----------------------------END MODULES DATA----------------------------");
}
