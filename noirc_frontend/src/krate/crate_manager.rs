use std::{collections::HashMap, fmt::Debug};

use super::crate_unit::{ModID, VirtualPath, CrateUnit};

/// Each crate is assigned an ID
/// The local crate is always assigned an ID of 0;
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CrateID(usize);
#[derive(Debug, Clone)]
pub struct CrateName(String);

/// The local crate will always have a CrateID of zero
pub const LOCAL_CRATE_ID : CrateID = CrateID(0); 

/// A project can have many crates. There is a local crate which is the crate being compiled, the dependencies and also transient dependencies
#[derive(Debug)]
pub struct CrateManager<Module : Debug> {
    name_to_crate : HashMap<String, CrateID>,
    /// Consists of the local crate and dependencies
    crates : Vec<CrateUnit<Module>>
}

impl<Module : Debug> CrateManager<Module> {
    pub fn new() -> CrateManager<Module> {

        CrateManager {
            name_to_crate : HashMap::new(),
            crates : Vec::new()
        }
    }

    pub fn debug_crate_name(&self) {
        let vec: Vec<_> = self.name_to_crate.keys().collect();
        dbg!(vec);
    }
    pub fn debug_virtual_paths(&self) {
        for krate in self.crates().iter() {
            krate.debug_virtual_paths()
        }
    }

    pub fn with_local_crate(local_crate : CrateUnit<Module>) -> CrateManager<Module> {

        let mut crate_system = CrateManager {
            name_to_crate : HashMap::new(),
            crates : Vec::new()
        };

        crate_system.insert_crate("crate".to_owned(), local_crate);
        
        crate_system
    }
    pub fn add_local_crate(&mut self, local_crate : CrateUnit<Module>) {

        assert!(self.crates.len() == 0, "ice: the local crate should be the first crate added");

        let id = self.insert_crate("crate".to_owned(), local_crate);
        assert!(id == LOCAL_CRATE_ID, "ice: local crate id is always {:?}, but it was found to be {:?}", LOCAL_CRATE_ID, id);
    }

    /// Inserts a new crate into the dependency graph for the local crate 
    /// This will panic, if you try to overwrite a crate name
    pub fn add_dep(&mut self, crate_name : String, module_system : CrateUnit<Module>)  -> CrateID{
        assert!(self.crates.len() >= 1, "the local crate should be the first crate added {}", self.crates.len());
        self.insert_crate(crate_name, module_system)
    }
    pub fn local_crate(&self) -> Option<&CrateUnit<Module>>{
        self.crates.first()
    }

    pub fn crates_mut(&mut self) -> &mut Vec<CrateUnit<Module>> {
        &mut self.crates
    }
    pub fn crates(&self) -> &Vec<CrateUnit<Module>> {
        &self.crates
    }
    
    pub fn crate_ids(&self) -> Vec<CrateID> {
        self.name_to_crate.values().copied().collect()
    }

    fn insert_crate(&mut self, crate_name : String, module_system : CrateUnit<Module>) -> CrateID {


        if self.crate_exists(&crate_name) {
            panic!("Compiler Error: Cannot overwrite a crate that already exists")
        }

        let crate_id = CrateID(self.crates.len());

        self.crates.push(module_system);

        self.name_to_crate.insert(crate_name, crate_id);

        crate_id
    }

    pub fn get_crate_with_id(&self, crate_id : CrateID) -> Option<&CrateUnit<Module>> {
        self.crates.get(crate_id.0)
    }
    pub fn get_mut_crate_with_id(&mut self, crate_id : CrateID) -> Option<&mut CrateUnit<Module>> {
        self.crates.get_mut(crate_id.0)
    }

    pub fn get_crate_with_name(&self, name: &str) -> Option<&CrateUnit<Module>> {
        let crate_id = self.name_to_crate.get(name)?;
        self.get_crate_with_id(*crate_id)
    }
    
    // Currently we only have main crate as a binary.
    // With workspaces, there would need to be something to classify them and store their classification
    pub fn get_all_libraries(&self) -> Option<Vec<&CrateUnit<Module>>> {
        let lib_crate_names : Vec<_> = self.name_to_crate.keys().
        filter(|crate_name| *crate_name != "main").collect();
    
        let mut crates = Vec::new();
        for lib_crate in lib_crate_names.iter() {
            crates.push(self.get_crate_with_name(lib_crate)?)
        }
        Some(crates)
    }

    pub fn crate_exists(&self, name : &str) -> bool {
        self.get_crate_with_name(name).is_some()
    }

    pub fn find_module(&self, path : VirtualPath) -> Option<(ModID, CrateID, &Module)> {        
        let (krate, mod_path) = path.segments();
        
        // First find the crate

        let c_id = self.name_to_crate.get(krate)?;
        let krate = self.crates.get(c_id.0)?;

        
        // Now find the module
        krate.debug_virtual_paths();
        let (mod_id, module) = krate.get_module_via_path(&mod_path)?;

        Some((mod_id, *c_id, module))
    }
}
