use acvm::acir::circuit::Circuit;
use acvm::Language;
use fm::FileType;
use noirc_abi::Abi;
use noirc_errors::{DiagnosableError, Reporter};
use noirc_evaluator::create_circuit;
use noirc_frontend::graph::{CrateId, CrateName, CrateType, LOCAL_CRATE};
use noirc_frontend::hir::def_map::CrateDefMap;
use noirc_frontend::hir::Context;
use noirc_frontend::monomorphisation::monomorphise;
use noirc_frontend::partialevaluator::evaluate;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug)]
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
        driver.into_compiled_program(np_language, false)
    }

    /// Compiles a file and returns true if compilation was successful
    ///
    /// This is used for tests.
    pub fn file_compiles(&mut self) -> bool {
        let mut errs = vec![];
        CrateDefMap::collect_defs(LOCAL_CRATE, &mut self.context, &mut errs);
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

    // NOTE: Maybe build could be skipped given that now it is a pass through method.
    /// Statically analyses the local crate
    pub fn build(&mut self) {
        self.analyse_crate()
    }

    fn analyse_crate(&mut self) {
        let mut errs = vec![];
        CrateDefMap::collect_defs(LOCAL_CRATE, &mut self.context, &mut errs);
        let mut error_count = 0;
        for errors in &errs {
            error_count += errors.errors.len();
            Reporter::with_diagnostics(errors.file_id, &self.context.file_manager, &errors.errors);
        }

        Reporter::finish(error_count);
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
        let abi = func_meta.into_abi(&self.context.def_interner);

        let program = monomorphise(main_function, self.context.def_interner);
        let program = evaluate(program);

        // Compile Program
        let circuit = match create_circuit(program, np_language, show_ssa) {
            Ok(circuit) => circuit,
            Err(err) => {
                // The FileId here will be the file id of the file with the main file
                // Errors will be shown at the callsite without a stacktrace
                Reporter::with_diagnostics(
                    err.location.file,
                    &self.context.file_manager,
                    &[err.to_diagnostic()],
                );
                Reporter::finish(1);
                unreachable!("reporter will exit before this point")
            }
        };

        CompiledProgram { circuit, abi: Some(abi) }
    }
}

impl Default for Driver {
    fn default() -> Self {
        Self::new(&Language::R1CS)
    }
}
