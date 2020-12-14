

mod virtual_path;
pub use virtual_path::*;

use std::{collections::HashMap, path::PathBuf};

/// Because we only allow one module per file, this will be equal to the FileID
/// This however, does not wrap the FileID, solely because we may extend the language
/// in the future and it would require a refactor
#[derive(Debug, Copy, Clone)]
pub struct ModID(usize);

/// It's important for the analyzer to know whether it is analyzing a binary or a library
/// At this level, there is no such thing as a `main.rs` for binaries.
/// This is enforced by nargo. The entry point is the root file. nargo also enforces the structure of projects by the way
/// XXX: Coming soon, a crate can only have a binary or a library file. For a project to have both, we introduce the notion of a workspace
#[derive(Debug)]
pub enum CrateType {
    LIBRARY, 
    BINARY,
}

/// A crate is a collection of modules, that must be compiled together.
/// The CrateUnit is used to manage all of the available modules in the crate
/// A crate is a compilation unit.

// - Parse from the root file, which will be the entry point for binaries
// - Save the directory of the root_file also, for relative paths 
// - Read up on rust-analyzer, to see how they manage crates/crate_data
// - We want to fix up the TypeChecker sooner rather than later and the layout of deps
#[derive(Debug)]
pub struct CrateUnit<Module> {
    root_file : PathBuf,
    root_dir : PathBuf,
    crate_type : CrateType,
    virtual_path_to_module : HashMap<VirtualPath, ModID>,
    module_name_to_mod_id : HashMap<String, ModID>,
    /// Currently modules are represented as `Programs` which can be converted into a SymbolTable
    /// XXX: we may deprecate the explicit symbol table altogether
    modules : Vec<Module>
}

impl<Module> CrateUnit<Module> {
    
    pub fn new(root_file : PathBuf, crate_type : CrateType) -> CrateUnit<Module> {
        CrateUnit::with(0, root_file, crate_type)
    }
    // A good heuristic for this is the number of files in the crate.
    // It is correct 100% of the time, while we allow only one file per module
    pub fn with(expected_modules : usize, root_file : PathBuf, crate_type : CrateType) -> CrateUnit<Module> {
        let mut root_dir = root_file.clone();
        root_dir.pop();
        CrateUnit {
            root_file,
            root_dir,
            crate_type,
            module_name_to_mod_id : HashMap::with_capacity(expected_modules),
            modules: Vec::with_capacity(expected_modules),
            virtual_path_to_module: HashMap::with_capacity(expected_modules),
        }
    }

    pub fn modules_mut(&mut self) -> &mut Vec<Module> {
        &mut self.modules
    }
 
    pub fn modules(&self) -> &Vec<Module> {
        &self.modules
    }
    pub fn module_ids(&self) -> Vec<ModID> {
        self.module_name_to_mod_id.values().copied().collect()
    }

    pub fn get_module(&self, mod_id : ModID) -> Option<&Module> {
        self.modules.get(mod_id.0)
    }
    pub fn get_module_with_name(&self, mod_name : &str) -> Option<&Module> {
        let mod_id = self.module_name_to_mod_id.get(mod_name)?;
        self.get_module(*mod_id)
    }
    pub fn get_mut_module(&mut self, mod_id : ModID) -> Option<&mut Module> {
        self.modules.get_mut(mod_id.0)
    }

    pub fn get_module_via_path(&self, virtual_path : &VirtualPath) -> Option<(ModID, &Module)> {
        let mod_id = self.virtual_path_to_module.get(virtual_path)?;
        Some((*mod_id, self.get_module(*mod_id)?))
    }
    
    pub fn insert_module(&mut self, file_path : PathBuf, mod_name : String, module : Module) -> ModID{
        
        let relative_path = pathdiff::diff_paths(file_path, self.root_file.to_path_buf()).unwrap();

        let mod_id = ModID(self.modules.len());

        self.modules.push(module);

        // virtualise path before placing it in the module system
        // The only time we access a module, is through import statements
        // To which we use it's virtual path
        self.virtual_path_to_module.insert(VirtualPath::from_relative_path(relative_path), mod_id);

        // Insert map for module name
        self.module_name_to_mod_id.insert(mod_name, mod_id);

        mod_id
    }
}
