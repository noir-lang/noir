//! The noir compiler is separated into the following passes which are listed
//! in order in square brackets. The inputs and outputs of each pass are also given:
//!
//! Source file -[Lexing]-> Tokens -[Parsing]-> Ast -[Name Resolution]-> Hir -[Type Checking]-> Hir -[Monomorphization]-> Monomorphized Ast
//!
//! After the monomorphized ast is created, it is passed to the noirc_evaluator crate to convert it to SSA form,
//! perform optimizations, convert to ACIR and eventually prove/verify the program.
use std::path::PathBuf;

use fm::{FileManager, FileType};
use hir::{def_map::CrateDefMap, Context};
use noirc_frontend::graph::{CrateGraph, CrateType};
use noirc_frontend::hir::{self, def_map::ModuleDefId};

// XXX: This is another sandbox test
fn main() {
    // File Manager
    let mut fm = FileManager::default();
    //
    // Add root file to file manager
    let dir_path: PathBuf = PathBuf::from("example_project/lib.nr");
    let root_file_id = fm.add_file(&dir_path, FileType::Root).unwrap();

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
    let mut errors = vec![];
    CrateDefMap::collect_defs(crate_id, &mut context, &mut errors);
    assert_eq!(errors, vec![]);
    let def_map = context.def_map(crate_id).unwrap();

    // Get root module
    let root = def_map.root();
    let module = def_map.modules().get(root.0).unwrap();
    for def_id in module.value_definitions() {
        let func_id = match def_id {
            ModuleDefId::FunctionId(func_id) => func_id,
            _ => unreachable!(),
        };

        // Get the HirFunction for that Id
        let hir = context.def_interner.function(&func_id);
        println!("func hir is {:?}", hir);
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
    //     println!("-----------------------------Children for module at position {}----------------------------", i);
    //     for (child_name, child_id) in &module_data.children {
    //         println!("{:?} is a child module with id {:?}", child_name, child_id);
    //     }
    //     println!("scope is : {:?}", module_data.scope);
    //     println!("origin is : {:?}", module_data.origin);
    //     println!("-----------------------------End Data for module at position {}----------------------------", i);
    // }
    // println!("-----------------------------END MODULES DATA----------------------------");
}
