use acvm::acir::circuit::Circuit;

use acvm::Language;
use fm::FileType;
use noirc_abi::Abi;
use noirc_errors::{DiagnosableError, ReportedError, Reporter};
use noirc_evaluator::create_circuit;
use noirc_frontend::graph::{CrateId, CrateName, CrateType, LOCAL_CRATE};
use noirc_frontend::hir::def_map::CrateDefMap;
use noirc_frontend::hir::Context;
use noirc_frontend::monomorphization::monomorphize;
use noirc_frontend::node_interner::FuncId;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub struct Driver {
    context: Context,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompiledProgram {
    pub circuit: Circuit,
    pub abi: Option<noirc_abi::Abi>,
}

impl Driver {
    pub fn new(np_language: &acvm::Language) -> Self {
        let mut driver = Driver { context: Context::default() };
        driver.context.def_interner.set_language(np_language);
        driver
    }

    // This is here for backwards compatibility
    // with the restricted version which only uses one file
    pub fn compile_file(root_file: PathBuf, np_language: acvm::Language) -> CompiledProgram {
        let mut driver = Driver::new(&np_language);
        driver.create_local_crate(root_file, CrateType::Binary);

        driver
            .into_compiled_program(np_language, false, false)
            .unwrap_or_else(|_| std::process::exit(1))
    }

    /// Compiles a file and returns true if compilation was successful
    ///
    /// This is used for tests.
    pub fn file_compiles(&mut self) -> bool {
        let mut errs = vec![];
        CrateDefMap::collect_defs(LOCAL_CRATE, &mut self.context, &mut errs);
        for errors in &errs {
            Reporter::with_diagnostics(
                Some(errors.file_id),
                &self.context.file_manager,
                &errors.errors,
                false,
            );
        }
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
    pub fn check_crate(&mut self, allow_warnings: bool) -> Result<(), ReportedError> {
        let mut errs = vec![];
        CrateDefMap::collect_defs(LOCAL_CRATE, &mut self.context, &mut errs);
        let mut error_count = 0;
        for errors in &errs {
            error_count += Reporter::with_diagnostics(
                Some(errors.file_id),
                &self.context.file_manager,
                &errors.errors,
                allow_warnings,
            );
        }

        Reporter::finish(error_count)
    }

    pub fn compute_abi(&self) -> Option<Abi> {
        let local_crate = self.context.def_map(LOCAL_CRATE).unwrap();

        let main_function = local_crate.main_function()?;

        let func_meta = self.context.def_interner.function_meta(&main_function);

        let abi = func_meta.into_abi(&self.context.def_interner);
        Some(abi)
    }

    pub fn into_compiled_program(
        mut self,
        np_language: acvm::Language,
        show_ssa: bool,
        allow_warnings: bool,
    ) -> Result<CompiledProgram, ReportedError> {
        self.check_crate(allow_warnings)?;
        self.compile_no_check(np_language, show_ssa, allow_warnings, None)
    }

    /// Compile the current crate. Assumes self.check_crate is called beforehand!
    #[allow(deprecated)]
    pub fn compile_no_check(
        &self,
        np_language: acvm::Language,
        show_ssa: bool,
        allow_warnings: bool,
        // Optional override to provide a different `main` function to start execution
        main_function: Option<FuncId>,
    ) -> Result<CompiledProgram, ReportedError> {
        // Check the crate type
        // We don't panic here to allow users to `evaluate` libraries
        // which will do nothing
        if self.context.crate_graph[LOCAL_CRATE].crate_type != CrateType::Binary {
            println!("cannot compile crate into a program as the local crate is not a binary. For libraries, please use the build command");
            std::process::exit(1);
        };

        // Find the local crate, one should always be present
        let local_crate = self.context.def_map(LOCAL_CRATE).unwrap();

        // All Binaries should have a main function
        let main_function = main_function.unwrap_or_else(|| {
            local_crate.main_function().expect("cannot compile a program with no main function")
        });

        // Create ABI for main function
        let func_meta = self.context.def_interner.function_meta(&main_function);
        let abi = func_meta.into_abi(&self.context.def_interner);

        let program = monomorphize(main_function, &self.context.def_interner);

        let blackbox_supported = acvm::default_is_blackbox_supported(np_language.clone());
        match create_circuit(program, np_language, blackbox_supported, show_ssa) {
            Ok(circuit) => Ok(CompiledProgram { circuit, abi: Some(abi) }),
            Err(err) => {
                // The FileId here will be the file id of the file with the main file
                // Errors will be shown at the call site without a stacktrace
                let file_id = err.location.map(|loc| loc.file);
                let error = &[err.to_diagnostic()];
                let files = &self.context.file_manager;

                let error_count = Reporter::with_diagnostics(file_id, files, error, allow_warnings);
                Reporter::finish(error_count)?;
                Err(ReportedError)
            }
        }
    }

    /// Returns a list of all functions in the current crate marked with #[test]
    /// whose names contain the given pattern string. An empty pattern string
    /// will return all functions marked with #[test].
    pub fn get_all_test_functions_in_crate_matching(&self, pattern: &str) -> Vec<FuncId> {
        let interner = &self.context.def_interner;
        interner
            .get_all_test_functions()
            .filter_map(|id| interner.function_name(&id).contains(pattern).then_some(id))
            .collect()
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
