use acvm::acir::circuit::Circuit;
use fm::FileType;
use noirc_abi::Abi;
use noirc_errors::DiagnosableError;
use noirc_errors::Reporter;
use noirc_evaluator::Evaluator;
use noirc_frontend::graph::{CrateId, CrateName, CrateType, LOCAL_CRATE};
use noirc_frontend::hir::def_map::CrateDefMap;
use noirc_frontend::hir::Context;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Driver {
    context: Context,
}
pub struct CompiledProgram {
    pub circuit: Circuit,
    pub abi: Option<noirc_abi::Abi>,
}

impl Driver {
    pub fn new() -> Self {
        Driver { context: Context::default() }
    }

    // This is here for backwards compatibility
    // with the restricted version which only uses one file
    pub fn compile_file(root_file: PathBuf, np_language: acvm::Language) -> CompiledProgram {
        let mut driver = Driver::new();
        driver.create_local_crate(root_file, CrateType::Binary);
        driver.into_compiled_program(np_language, false)
    }

    /// Compiles a file and returns true if compilation was successful
    ///
    /// This is used for tests.
    pub fn file_compiles<P: AsRef<Path>>(root_file: P) -> bool {
        let mut driver = Driver::new();
        driver.create_local_crate(root_file, CrateType::Binary);
        driver.add_std_lib();
        let mut errs = vec![];
        CrateDefMap::collect_defs(LOCAL_CRATE, &mut driver.context, &mut errs);
        for errors in &errs {
            dbg!(errors);
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
            panic!("crates cannot depend on binaries. {:?} is a binary crate", crate_name)
        }

        self.context
            .crate_graph
            .add_dep(this_crate, crate_name, depends_on)
            .expect("cyclic dependency triggered");
    }

    /// Adds the standard library to the dep graph
    /// and statically analyses the local crate
    pub fn build(&mut self) {
        self.add_std_lib();

        self.analyse_crate()
    }

    fn analyse_crate(&mut self) {
        let mut errs = vec![];
        CrateDefMap::collect_defs(LOCAL_CRATE, &mut self.context, &mut errs);
        let mut error_count = 0;
        for errors in &errs {
            error_count += errors.errors.len();
            Reporter::with_diagnostics(
                errors.file_id.as_usize(),
                &self.context.file_manager,
                &errors.errors,
            );
        }

        Reporter::finish(error_count);
    }

    pub fn compute_abi(&self) -> Option<Abi> {
        let local_crate = self.context.def_map(LOCAL_CRATE).unwrap();

        let main_function = local_crate.main_function()?;

        let func_meta = self.context.def_interner.function_meta(&main_function);
        let abi = func_meta.parameters.into_abi(&self.context.def_interner);

        Some(abi)
    }

    pub fn into_compiled_program(
        mut self,
        np_language: acvm::Language,
        show_ssa: bool,
    ) -> CompiledProgram {
        self.build();
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
        let main_function =
            local_crate.main_function().expect("cannot compile a program with no main function");

        // Create ABI for main function
        let func_meta = self.context.def_interner.function_meta(&main_function);
        let abi = func_meta.parameters.into_abi(&self.context.def_interner);

        let evaluator = Evaluator::new(main_function, &self.context);

        // Compile Program
        let circuit = match evaluator.compile(np_language, show_ssa) {
            Ok(circuit) => circuit,
            Err(err) => {
                // The FileId here will be the file id of the file with the main file
                // Errors will be shown at the callsite without a stacktrace
                Reporter::with_diagnostics(
                    err.file,
                    &self.context.file_manager,
                    &[err.to_diagnostic()],
                );
                Reporter::finish(1);
                unreachable!("reporter will exit before this point")
            }
        };

        CompiledProgram { circuit, abi: Some(abi) }
    }

    /// XXX: It is sub-optimal to add the std as a regular crate right now because
    /// we have no way to determine whether a crate has been compiled already.
    /// XXX: We Ideally need a way to check if we've already compiled a crate and not re-compile it
    pub fn add_std_lib(&mut self) {
        let path_to_std_lib_file = path_to_stdlib().join("lib.nr");

        let std_crate_id = self.create_non_local_crate(path_to_std_lib_file, CrateType::Library);

        let name = CrateName::new("std").unwrap();

        let crate_ids: Vec<_> = self
            .context
            .crate_graph
            .iter_keys()
            .filter(|crate_id| *crate_id != std_crate_id)
            .collect();
        // Add std as a crate dependency to every other crate
        for crate_id in crate_ids {
            self.context
                .crate_graph
                .add_dep(crate_id, name.clone(), std_crate_id)
                .expect("ice: cyclic error triggered with std library");
        }
    }
}

impl Default for Driver {
    fn default() -> Self {
        Self::new()
    }
}

fn path_to_stdlib() -> PathBuf {
    dirs::config_dir().unwrap().join("noir-lang").join("std")
}
