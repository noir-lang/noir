
use std::path::{Path, PathBuf};
use fm::FileManager;
use noirc_errors::Reporter;
use noirc_errors::DiagnosableError;
use noirc_frontend::{hir::{crate_def_map::CrateDefMap, crate_graph::{CrateGraph, CrateId, CrateName, CrateType, LOCAL_CRATE}, lower::def_interner::FuncId}};
use noirc_frontend::hir::Context;
use noir_evaluator::Evaluator;
use noirc_frontend::ast::Type;
use acir::circuit::Circuit;

#[derive(Debug)]
pub struct Driver {
    context : Context,
}
pub struct CompiledProgram {
    pub circuit : Circuit,
    pub num_witnesses : usize, 
    pub num_public_inputs : usize, 
    pub abi : Option<Vec<(String, Type)>>
}

impl Driver{
    pub fn new() -> Self {
        Driver {context : Context::default()}
    }

    // This is here for backwards compatibility.
    pub fn compile_file(root_file : PathBuf) -> CompiledProgram {
        let mut driver = Driver::new();
        driver.create_local_crate(root_file, CrateType::Binary);
        driver.into_compiled_program()
    }

    /// Adds the File with the local crate root to the file system 
    /// and adds the local crate to the graph
    pub fn create_local_crate<P: AsRef<Path>>(&mut self, root_file : P, crate_type : CrateType) {
        
        let dir_path = root_file.as_ref().to_path_buf();
        let root_file_id = self.context.file_manager().add_file(&dir_path).unwrap();
        
        let crate_id = self.context.crate_graph.add_crate_root(crate_type, root_file_id);
        assert!(crate_id == LOCAL_CRATE);
    }

    /// Creates a Non Local Crate. A Non Local Crate is any crate which is the not the crate that 
    /// the compiler is compiling.  
    pub fn create_non_local_crate<P: AsRef<Path>>(&mut self, root_file : P) -> CrateId {
        let dir_path = root_file.as_ref().to_path_buf();
        let root_file_id = self.context.file_manager().add_file(&dir_path).unwrap();
        
        // The first crate is always the local crate
        assert!(self.context.crate_graph.number_of_crates() != 0);

        self.context.crate_graph.add_crate_root(CrateType::Library, root_file_id)// You can only depend on libraries
    }

    /// Adds a edge in the crate graph for two crates
    pub fn add_dep(&mut self, this_crate : CrateId, depends_on : CrateId, crate_name : &str) {
        let crate_name = CrateName::new(crate_name).expect("crate name contains blacklisted characters, please remove");
        self.context.crate_graph.add_dep(this_crate, crate_name, depends_on).expect("cyclic dependency triggered");
    }

    pub fn build(&mut self) {
        self.add_std_lib();

        self.analyse_crate()
    }

    fn analyse_crate(&mut self) {
        CrateDefMap::collect_defs(LOCAL_CRATE, &mut self.context);
        // XXX: We need to modify the new Analyser to propagate errors
        // if let Err(errs) =  analyser::check_crates(&mut self.crate_manager) {
        //     let diagnostics : Vec<_> = errs.into_iter().map(|err| err.to_diagnostic()).collect();
        //     Reporter::with_diagnostics(&self.file_manager, &diagnostics);
        //     std::process::exit(1);
        // }
    }

    pub fn into_compiled_program(&mut self) -> CompiledProgram{
        self.build();

              
        // First find the local crate 
        // There is always a local crate
        let local_crate = self.context.def_map(LOCAL_CRATE).unwrap();
        let file_id = local_crate.root_file_id().as_usize();
        
        // Check the crate type
        // We don't panic here to allow users to `evaluate` libraries
        // which will do nothing
        if self.context.crate_graph()[LOCAL_CRATE].crate_type != CrateType::Binary {
            println!("cannot compile crate into a program as the local crate is not a binary. For libraries, please use the build flag");
            std::process::exit(1);   
        };

        // All Binaries should have a main function
        let main_function = local_crate.main_function().expect("cannot compile a program with no main function");

        // Create ABi for main function
        let abi = self.func_to_abi(main_function);

        let evaluator = Evaluator::new(file_id, main_function, &self.context);

        // Compile Program
        let (circuit, num_witnesses, num_public_inputs) = match evaluator.compile() {
            Ok((circuit, num_witnesses, num_public_inputs)) => (circuit, num_witnesses, num_public_inputs),
            Err(err) => {
                Reporter::with_diagnostics(&self.context.file_manager(), &vec![err.to_diagnostic()]);
                std::process::exit(1);
            }
        };
        
        CompiledProgram {
            circuit,
            num_witnesses,
            num_public_inputs,
            abi :Some(abi)
        }
    }

    /// XXX: It is sub-optimal to add the std as a regular crate right now because
    /// we have no way to determine whether a crate has been compiled already.
    /// XXX: We Ideally need a way to check if we've already compiled a crate and not re-compile it
    pub fn add_std_lib(&mut self){
        let path_to_std_lib_file = path_to_stdlib().join("lib.nr");

        let std_crate_id = self.create_non_local_crate(path_to_std_lib_file);
        
        let name = CrateName::new("std").unwrap();

        let crate_ids : Vec<_> = self.context.crate_graph.iter_keys().filter(|crate_id| *crate_id != std_crate_id).collect();
        // Add std as a crate dependency to every other crate
        for crate_id in crate_ids {
            self.context.crate_graph.add_dep(crate_id, name.clone(), std_crate_id).expect("ice: cyclic error triggered with std library");
        }
    }
    
    /// Creates an ABI from a function
    fn func_to_abi(&self, func_id : FuncId) -> Vec<(String, Type)>{
        let func_meta = self.context.def_interner.function_meta(func_id);

        func_meta.parameters.into_iter().map(|param| {
            let (param_id, param_type) = (param.0, param.1);
            let param_name = self.context.def_interner.id_name(param_id);
            (param_name, param_type)
        }).collect()

    }
}

fn path_to_stdlib() -> PathBuf {
    dirs::config_dir().unwrap().join("noir-lang").join("std")
}
