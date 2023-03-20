#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]

use acvm::Language;
use clap::Args;
use fm::FileType;
use iter_extended::{try_btree_map, try_vecmap};
use noirc_abi::FunctionSignature;
use noirc_errors::{reporter, ReportedError};
use noirc_evaluator::create_circuit;
use noirc_frontend::graph::{CrateId, CrateName, CrateType, LOCAL_CRATE};
use noirc_frontend::hir::def_map::{Contract, CrateDefMap};
use noirc_frontend::hir::Context;
use noirc_frontend::monomorphization::monomorphize;
use noirc_frontend::node_interner::FuncId;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

mod contract;
mod program;

pub use contract::CompiledContract;
pub use program::CompiledProgram;

pub struct Driver {
    context: Context,
    language: Language,
}

#[derive(Args, Clone, Debug, Serialize, Deserialize)]
pub struct CompileOptions {
    /// Emit debug information for the intermediate SSA IR
    #[arg(short, long)]
    pub show_ssa: bool,

    /// Issue a warning for each unused variable instead of an error
    #[arg(short, long)]
    pub allow_warnings: bool,

    /// Display output of `println` statements during tests
    #[arg(long)]
    pub show_output: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self { show_ssa: false, allow_warnings: false, show_output: true }
    }
}

impl Driver {
    pub fn new(np_language: &Language) -> Self {
        let mut driver = Driver { context: Context::default(), language: np_language.clone() };
        driver.context.def_interner.set_language(np_language);
        driver
    }

    // This is here for backwards compatibility
    // with the restricted version which only uses one file
    pub fn compile_file(
        root_file: PathBuf,
        np_language: acvm::Language,
    ) -> Result<CompiledProgram, ReportedError> {
        let mut driver = Driver::new(&np_language);
        driver.create_local_crate(root_file, CrateType::Binary);
        driver.compile_main(&CompileOptions::default())
    }

    /// Compiles a file and returns true if compilation was successful
    ///
    /// This is used for tests.
    pub fn file_compiles(&mut self) -> bool {
        let mut errs = vec![];
        CrateDefMap::collect_defs(LOCAL_CRATE, &mut self.context, &mut errs);
        reporter::report_all(&self.context.file_manager, &errs, false);
        errs.is_empty()
    }

    /// Adds the File with the local crate root to the file system
    /// and adds the local crate to the graph
    /// XXX: This may pose a problem with workspaces, where you can change the local crate and where
    /// we have multiple binaries in one workspace
    /// A Fix would be for the driver instance to store the local crate id.
    // Granted that this is the only place which relies on the local crate being first
    pub fn create_local_crate<P: AsRef<Path>>(
        &mut self,
        root_file: P,
        crate_type: CrateType,
    ) -> CrateId {
        let dir_path = root_file.as_ref().to_path_buf();
        let root_file_id = self.context.file_manager.add_file(&dir_path, FileType::Root).unwrap();

        let crate_id = self.context.crate_graph.add_crate_root(crate_type, root_file_id);

        assert!(crate_id == LOCAL_CRATE);

        LOCAL_CRATE
    }

    /// Creates a Non Local Crate. A Non Local Crate is any crate which is the not the crate that
    /// the compiler is compiling.
    pub fn create_non_local_crate<P: AsRef<Path>>(
        &mut self,
        root_file: P,
        crate_type: CrateType,
    ) -> CrateId {
        let dir_path = root_file.as_ref().to_path_buf();
        let root_file_id = self.context.file_manager.add_file(&dir_path, FileType::Root).unwrap();

        // The first crate is always the local crate
        assert!(self.context.crate_graph.number_of_crates() != 0);

        // You can add any crate type to the crate graph
        // but you cannot depend on Binaries
        self.context.crate_graph.add_crate_root(crate_type, root_file_id)
    }

    /// Adds a edge in the crate graph for two crates
    pub fn add_dep(&mut self, this_crate: CrateId, depends_on: CrateId, crate_name: &str) {
        let crate_name = CrateName::new(crate_name)
            .expect("crate name contains blacklisted characters, please remove");

        // Cannot depend on a binary
        if self.context.crate_graph.crate_type(depends_on) == CrateType::Binary {
            panic!("crates cannot depend on binaries. {crate_name:?} is a binary crate")
        }

        self.context
            .crate_graph
            .add_dep(this_crate, crate_name, depends_on)
            .expect("cyclic dependency triggered");
    }

    /// Propagates a given dependency to every other crate.
    pub fn propagate_dep(&mut self, dep_to_propagate: CrateId, dep_to_propagate_name: &CrateName) {
        let crate_ids: Vec<_> = self
            .context
            .crate_graph
            .iter_keys()
            .filter(|crate_id| *crate_id != dep_to_propagate)
            .collect();

        for crate_id in crate_ids {
            self.context
                .crate_graph
                .add_dep(crate_id, dep_to_propagate_name.clone(), dep_to_propagate)
                .expect("ice: cyclic error triggered with std library");
        }
    }

    /// Run the lexing, parsing, name resolution, and type checking passes,
    /// returning Err(FrontendError) and printing any errors that were found.
    pub fn check_crate(&mut self, options: &CompileOptions) -> Result<(), ReportedError> {
        let mut errs = vec![];
        CrateDefMap::collect_defs(LOCAL_CRATE, &mut self.context, &mut errs);
        let error_count =
            reporter::report_all(&self.context.file_manager, &errs, options.allow_warnings);
        reporter::finish_report(error_count)
    }

    pub fn compute_function_signature(&self) -> Option<FunctionSignature> {
        let local_crate = self.context.def_map(LOCAL_CRATE).unwrap();

        let main_function = local_crate.main_function()?;

        let func_meta = self.context.def_interner.function_meta(&main_function);

        Some(func_meta.into_function_signature(&self.context.def_interner))
    }

    /// Run the frontend to check the crate for errors then compile the main function if there were none
    pub fn compile_main(
        &mut self,
        options: &CompileOptions,
    ) -> Result<CompiledProgram, ReportedError> {
        self.check_crate(options)?;
        let main = match self.main_function() {
            Ok(m) => m,
            Err(e) => {
                println!("cannot compile a program with no main function");
                return Err(e);
            }
        };
        self.compile_no_check(options, main)
    }

    /// Run the frontend to check the crate for errors then compile all contracts if there were none
    pub fn compile_contracts(
        &mut self,
        options: &CompileOptions,
    ) -> Result<Vec<CompiledContract>, ReportedError> {
        self.check_crate(options)?;
        let contracts = self.get_all_contracts();
        try_vecmap(contracts, |contract| self.compile_contract(contract, options))
    }

    /// Compile all of the functions associated with a Noir contract.
    fn compile_contract(
        &self,
        contract: Contract,
        options: &CompileOptions,
    ) -> Result<CompiledContract, ReportedError> {
        let functions = try_btree_map(&contract.functions, |function| {
            let function_name = self.function_name(*function).to_owned();

            self.compile_no_check(options, *function).map(|program| (function_name, program))
        })?;

        Ok(CompiledContract { name: contract.name, functions })
    }

    /// Returns the FuncId of the 'main' funciton.
    /// - Expects check_crate to be called beforehand
    /// - Panics if no main function is found
    pub fn main_function(&self) -> Result<FuncId, ReportedError> {
        // Find the local crate, one should always be present
        let local_crate = self.context.def_map(LOCAL_CRATE).unwrap();

        // Check the crate type
        // We don't panic here to allow users to `evaluate` libraries which will do nothing
        if self.context.crate_graph[LOCAL_CRATE].crate_type != CrateType::Binary {
            println!("cannot compile crate into a program as the local crate is not a binary. For libraries, please use the check command");
            return Err(ReportedError);
        };

        // All Binaries should have a main function
        match local_crate.main_function() {
            Some(func_id) => Ok(func_id),
            None => Err(ReportedError),
        }
    }

    /// Compile the current crate. Assumes self.check_crate is called beforehand!
    #[allow(deprecated)]
    pub fn compile_no_check(
        &self,
        options: &CompileOptions,
        main_function: FuncId,
    ) -> Result<CompiledProgram, ReportedError> {
        let program = monomorphize(main_function, &self.context.def_interner);

        let np_language = self.language.clone();
        let blackbox_supported = acvm::default_is_black_box_supported(np_language.clone());

        match create_circuit(
            program,
            np_language,
            blackbox_supported,
            options.show_ssa,
            options.show_output,
        ) {
            Ok((circuit, abi)) => Ok(CompiledProgram { circuit, abi }),
            Err(err) => {
                // The FileId here will be the file id of the file with the main file
                // Errors will be shown at the call site without a stacktrace
                let file = err.location.map(|loc| loc.file);
                let files = &self.context.file_manager;
                let error = reporter::report(files, &err.into(), file, options.allow_warnings);
                reporter::finish_report(error as u32)?;
                Err(ReportedError)
            }
        }
    }

    /// Returns a list of all functions in the current crate marked with #[test]
    /// whose names contain the given pattern string. An empty pattern string
    /// will return all functions marked with #[test].
    pub fn get_all_test_functions_in_crate_matching(&self, pattern: &str) -> Vec<FuncId> {
        let interner = &self.context.def_interner;
        self.context
            .def_map(LOCAL_CRATE)
            .expect("The local crate should be analyzed already")
            .get_all_test_functions(interner)
            .filter_map(|id| interner.function_name(&id).contains(pattern).then_some(id))
            .collect()
    }

    /// Return a Vec of all `contract` declarations in the source code and the functions they contain
    pub fn get_all_contracts(&self) -> Vec<Contract> {
        self.context
            .def_map(LOCAL_CRATE)
            .expect("The local crate should be analyzed already")
            .get_all_contracts()
    }

    pub fn function_name(&self, id: FuncId) -> &str {
        self.context.def_interner.function_name(&id)
    }
}

impl Default for Driver {
    fn default() -> Self {
        Self::new(&Language::R1CS)
    }
}
