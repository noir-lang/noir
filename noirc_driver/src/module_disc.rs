use std::path::PathBuf;
use super::dir_util;
use dir_util::{FILE_EXTENSION, MOD_FILE};
use noirc_frontend::Program;
use crate::Driver;
use nargo::CrateUnit;

// We need this mod type because not every file can declare a new module.
// If every file could declare a module, which is what Rust does, then we can remove this.
/// A module is either a single file or it is a folder with a `mod.nr` file
/// In the latter case, we call that a sub-module
#[derive(Debug, Clone)]
pub enum ModType { 
    Module(PathBuf),
    SubModule(PathBuf),
}

/// There are only two ways to discover modules:
/// - Manually adding it's path
/// - Recursive discovery through the mod key word;
struct ModuleDiscovery;

// Recursively parse a module given a
pub fn recursively_parse(driver : &mut Driver, module_system : &mut CrateUnit<Program>, module : ModType) {

    match module {
        ModType::Module(path) => {
            let filename = path.file_name().unwrap().to_str().unwrap().to_owned();

            let src = std::fs::read_to_string(&path).expect(&format!("expected a file at path: {}", path.to_str().unwrap()));
            let (program, _) = driver.parse_file(&path, src);

            assert!(program.module_decls.is_empty(), "module declarations can only be put in a mod.nr file or a lib.nr file");
            
            module_system.insert_module(path, filename, program);
        },
        ModType::SubModule(path_to_mod_file) => {
            
            let dir_with_mod_file = path_to_mod_file.parent().unwrap();
            
            let file_as_string = std::fs::read_to_string(&path_to_mod_file).unwrap();
            
            let (program, _) = driver.parse_file(&path_to_mod_file, file_as_string);
            module_system.insert_module(path_to_mod_file.clone(), "mod".to_owned(),program.clone());
            // XXX: the analysed symbol table is ignored because the analysis will take in a symbol table instead of a program
            // To deal with type checking, we will tag each expr/stmt from the parser and create a sidetable
            // In the future, we can remove analysis and just parse. Unless we decide to allow code written in the mod file also.
            
            let path_str = dir_with_mod_file.to_str().unwrap();
            
            for module_name in program.module_decls.iter(){  
                find_and_recursively_parse(driver, module_system,&path_str, &module_name);
            };
        },
    }
}

fn find_and_recursively_parse(driver : &mut Driver, module_system : &mut CrateUnit<Program>, current_dir : &str, mod_name : &str) {

    let module = find_module(current_dir, mod_name);

    recursively_parse(driver, module_system, module)
}


fn find_module(current_dir : &str, mod_name : &str) -> ModType {
    let mod_path = search_for_module_def(current_dir, mod_name);

    let module = match mod_path {
        None => panic!("could not find the module {} under the directory {}", mod_name, current_dir),
        Some(module) => module
    };

    module
}

// Searches for name/mod.nr or name.nr
// This is a part of noirc
fn search_for_module_def(root_path : &str,name : &str) -> Option<ModType> {
    let dir_mod = dir_util::find_dir(root_path, name);
    let file_mod = dir_util::find_file(root_path, name, FILE_EXTENSION);

    // Check if we have a SubModule
    let submodule =match dir_mod {
        Some(dir) => {
            match dir_util::find_file(dir.clone(), MOD_FILE, FILE_EXTENSION) {
                Some(mod_file) => Some(ModType::SubModule(mod_file)),
                None => {
                    panic!("cannot find mod.nr file in directory, {:?}", dir.to_str())
                } 
            }
        },
        None => None
    };

    // Check if we have a Module(file)
    let module =  match file_mod {
        Some(file) => Some(ModType::Module(file)),
        None => None,
    };
    
    
    // We cannot simultaneously have both of them or none of them
    // XXX: We don't currently have error reporting for this module yet
    match (submodule, module) {
        (None, None) => panic!("cannot find a module named {} at path {}", name, root_path),
        (Some(_), Some(_)) => panic!("found both a file and folder named {}.XXX: Actually we should check for a mod file too", name),
        (Some(x),None) | (None, Some(x)) => Some(x) 
    }
}
