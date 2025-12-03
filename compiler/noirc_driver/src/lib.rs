#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]

use std::hash::BuildHasher;

use abi_gen::{abi_type_from_hir_type, value_from_hir_expression};
use acvm::acir::circuit::ExpressionWidth;
use acvm::compiler::MIN_EXPRESSION_WIDTH;
use clap::Args;
use fm::{FileId, FileManager};
use iter_extended::vecmap;
use noirc_abi::{AbiParameter, AbiType, AbiValue};
use noirc_errors::{CustomDiagnostic, DiagnosticKind};
use noirc_evaluator::brillig::BrilligOptions;
use noirc_evaluator::create_program;
use noirc_evaluator::errors::RuntimeError;
use noirc_evaluator::ssa::opt::{CONSTANT_FOLDING_MAX_ITER, INLINING_MAX_INSTRUCTIONS};
use noirc_evaluator::ssa::{
    SsaEvaluatorOptions, SsaLogging, SsaProgramArtifact, create_program_with_minimal_passes,
};
use noirc_frontend::debug::build_debug_crate_file;
use noirc_frontend::elaborator::{FrontendOptions, UnstableFeature};
use noirc_frontend::hir::Context;
use noirc_frontend::hir::def_map::{CrateDefMap, ModuleDefId, ModuleId};
use noirc_frontend::monomorphization::{
    errors::MonomorphizationError, monomorphize, monomorphize_debug,
};
use noirc_frontend::node_interner::{FuncId, GlobalId, TypeId};
use noirc_frontend::token::SecondaryAttributeKind;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;

mod abi_gen;
mod contract;
mod debug;
mod program;
mod stdlib;

use debug::filter_relevant_files;

pub use abi_gen::gen_abi;
pub use contract::{CompiledContract, CompiledContractOutputs, ContractFunction};
pub use debug::DebugFile;
pub use noirc_frontend::graph::{CrateId, CrateName};
pub use program::CompiledProgram;
pub use stdlib::{stdlib_nargo_toml_source, stdlib_paths_with_source};

const STD_CRATE_NAME: &str = "std";
const DEBUG_CRATE_NAME: &str = "__debug";

pub const GIT_COMMIT: &str = env!("GIT_COMMIT");
pub const GIT_DIRTY: &str = env!("GIT_DIRTY");
pub const NOIRC_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Version string that gets placed in artifacts that Noir builds. This is semver compatible.
/// Note: You can't directly use the value of a constant produced with env! inside a concat! macro.
pub const NOIR_ARTIFACT_VERSION_STRING: &str =
    concat!(env!("CARGO_PKG_VERSION"), "+", env!("GIT_COMMIT"));

#[derive(Args, Clone, Debug)]
pub struct CompileOptions {
    /// Specify the backend expression width that should be targeted
    #[arg(long, value_parser = parse_expression_width)]
    pub expression_width: Option<ExpressionWidth>,

    /// Generate ACIR with the target backend expression width.
    /// The default is to generate ACIR without a bound and split expressions after code generation.
    /// Activating this flag can sometimes provide optimizations for certain programs.
    #[arg(long, default_value = "false")]
    pub bounded_codegen: bool,

    /// Force a full recompilation.
    #[arg(long = "force")]
    pub force_compile: bool,

    /// Emit debug information for the intermediate SSA IR to stdout
    #[arg(long, hide = true)]
    pub show_ssa: bool,

    /// Only show SSA passes whose name contains the provided string.
    /// This setting takes precedence over `show_ssa` if it's not empty.
    #[arg(long, hide = true)]
    pub show_ssa_pass: Vec<String>,

    /// Emit source file locations when emitting debug information for the SSA IR to stdout.
    /// By default, source file locations won't be shown.
    #[arg(long, hide = true)]
    pub with_ssa_locations: bool,

    /// Only show the SSA and ACIR for the contract function with a given name.
    #[arg(long, hide = true)]
    pub show_contract_fn: Option<String>,

    /// Skip SSA passes whose name contains the provided string(s).
    #[arg(long, hide = true)]
    pub skip_ssa_pass: Vec<String>,

    /// Emit the unoptimized SSA IR to file.
    /// The IR will be dumped into the workspace target directory,
    /// under `[compiled-package].ssa.json`.
    #[arg(long, hide = true)]
    pub emit_ssa: bool,

    /// Only perform the minimum number of SSA passes.
    ///
    /// The purpose of this is to be able to debug fuzzing failures.
    /// It implies `--force-brillig`.
    #[arg(long, hide = true)]
    pub minimal_ssa: bool,

    /// Display debug prints during Brillig generation.
    #[arg(long, hide = true)]
    pub show_brillig: bool,

    /// Display Brillig opcodes with advisories, if any.
    #[arg(long, hide = true)]
    pub show_brillig_opcode_advisories: bool,

    /// Display the ACIR for compiled circuit, including the Brillig bytecode.
    #[arg(long)]
    pub print_acir: bool,

    /// Pretty print benchmark times of each code generation pass
    #[arg(long, hide = true)]
    pub benchmark_codegen: bool,

    /// Treat all warnings as errors
    #[arg(long, conflicts_with = "silence_warnings")]
    pub deny_warnings: bool,

    /// Suppress warnings
    #[arg(long, conflicts_with = "deny_warnings")]
    pub silence_warnings: bool,

    /// Outputs the monomorphized IR to stdout for debugging
    #[arg(long, hide = true)]
    pub show_monomorphized: bool,

    /// Insert debug symbols to inspect variables
    #[arg(long, hide = true)]
    pub instrument_debug: bool,

    /// Force Brillig output (for step debugging)
    #[arg(long, hide = true)]
    pub force_brillig: bool,

    /// Enable printing results of comptime evaluation: provide a path suffix
    /// for the module to debug, e.g. "package_name/src/main.nr"
    #[arg(long)]
    pub debug_comptime_in_file: Option<String>,

    /// Outputs the paths to any modified artifacts
    #[arg(long, hide = true)]
    pub show_artifact_paths: bool,

    /// Flag to turn off the compiler check for under constrained values.
    /// Warning: This can improve compilation speed but can also lead to correctness errors.
    /// This check should always be run on production code.
    #[arg(long)]
    pub skip_underconstrained_check: bool,

    /// Flag to turn off the compiler check for missing Brillig call constraints.
    /// Warning: This can improve compilation speed but can also lead to correctness errors.
    /// This check should always be run on production code.
    #[arg(long)]
    pub skip_brillig_constraints_check: bool,

    /// Flag to turn on extra Brillig bytecode to be generated to guard against invalid states in testing.
    #[arg(long, hide = true)]
    pub enable_brillig_debug_assertions: bool,

    /// Count the number of arrays that are copied in an unconstrained context for performance debugging
    #[arg(long)]
    pub count_array_copies: bool,

    /// Flag to turn on the lookback feature of the Brillig call constraints
    /// check, allowing tracking argument values before the call happens preventing
    /// certain rare false positives (leads to a slowdown on large rollout functions)
    #[arg(long)]
    pub enable_brillig_constraints_check_lookback: bool,

    /// Setting to decide on an inlining strategy for Brillig functions.
    /// A more aggressive inliner should generate larger programs but more optimized
    /// A less aggressive inliner should generate smaller programs
    #[arg(long, allow_hyphen_values = true, default_value_t = i64::MAX)]
    pub inliner_aggressiveness: i64,

    /// Maximum number of iterations to do in constant folding, as long as new values are being hoisted.
    /// A value of 0 effectively disables constant folding.
    #[arg(long, hide = true, allow_hyphen_values = true, default_value_t = CONSTANT_FOLDING_MAX_ITER)]
    pub constant_folding_max_iter: usize,

    /// Setting to decide the maximum weight threshold at which we designate a function
    /// as "small" and thus to always be inlined.
    #[arg(long, hide = true, allow_hyphen_values = true, default_value_t = INLINING_MAX_INSTRUCTIONS)]
    pub small_function_max_instructions: usize,

    /// Setting the maximum acceptable increase in Brillig bytecode size due to
    /// unrolling small loops. When left empty, any change is accepted as long
    /// as it required fewer SSA instructions.
    /// A higher value results in fewer jumps but a larger program.
    /// A lower value keeps the original program if it was smaller, even if it has more jumps.
    #[arg(long, hide = true, allow_hyphen_values = true)]
    pub max_bytecode_increase_percent: Option<i32>,

    /// Use pedantic ACVM solving, i.e. double-check some black-box function
    /// assumptions when solving.
    /// This is disabled by default.
    #[arg(long, default_value = "false")]
    pub pedantic_solving: bool,

    /// Skip reading files/folders from the root directory and instead accept the
    /// contents of `main.nr` through STDIN.
    ///
    /// The implicit package structure is:
    /// ```
    /// src/main.nr // STDIN
    /// Nargo.toml // fixed "bin" Nargo.toml
    /// ```
    #[arg(long, hide = true)]
    pub debug_compile_stdin: bool,

    /// Unstable features to enable for this current build.
    ///
    /// If non-empty, it disables unstable features required in crate manifests.
    #[arg(value_parser = clap::value_parser!(UnstableFeature))]
    #[clap(long, short = 'Z', value_delimiter = ',', conflicts_with = "no_unstable_features")]
    pub unstable_features: Vec<UnstableFeature>,

    /// Disable any unstable features required in crate manifests.
    #[arg(long, conflicts_with = "unstable_features")]
    pub no_unstable_features: bool,

    /// Used internally to avoid comptime println from producing output
    #[arg(long, hide = true)]
    pub disable_comptime_printing: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            expression_width: None,
            bounded_codegen: false,
            force_compile: false,
            show_ssa: false,
            show_ssa_pass: Vec::new(),
            with_ssa_locations: false,
            show_contract_fn: None,
            skip_ssa_pass: Vec::new(),
            emit_ssa: false,
            minimal_ssa: false,
            show_brillig: false,
            show_brillig_opcode_advisories: false,
            print_acir: false,
            benchmark_codegen: false,
            deny_warnings: false,
            silence_warnings: false,
            show_monomorphized: false,
            instrument_debug: false,
            force_brillig: false,
            debug_comptime_in_file: None,
            show_artifact_paths: false,
            skip_underconstrained_check: false,
            skip_brillig_constraints_check: false,
            enable_brillig_debug_assertions: false,
            count_array_copies: false,
            enable_brillig_constraints_check_lookback: false,
            inliner_aggressiveness: i64::MAX,
            constant_folding_max_iter: CONSTANT_FOLDING_MAX_ITER,
            small_function_max_instructions: INLINING_MAX_INSTRUCTIONS,
            max_bytecode_increase_percent: None,
            pedantic_solving: false,
            debug_compile_stdin: false,
            unstable_features: Vec::new(),
            no_unstable_features: false,
            disable_comptime_printing: false,
        }
    }
}

impl CompileOptions {
    pub fn as_ssa_options(&self, package_build_path: PathBuf) -> SsaEvaluatorOptions {
        SsaEvaluatorOptions {
            ssa_logging: if !self.show_ssa_pass.is_empty() {
                SsaLogging::Contains(self.show_ssa_pass.clone())
            } else if self.show_ssa {
                SsaLogging::All
            } else {
                SsaLogging::None
            },
            brillig_options: BrilligOptions {
                enable_debug_trace: self.show_brillig,
                enable_debug_assertions: self.enable_brillig_debug_assertions,
                enable_array_copy_counter: self.count_array_copies,
                show_opcode_advisories: self.show_brillig_opcode_advisories,
                layout: Default::default(),
            },
            print_codegen_timings: self.benchmark_codegen,
            expression_width: if self.bounded_codegen {
                self.expression_width.unwrap_or(DEFAULT_EXPRESSION_WIDTH)
            } else {
                ExpressionWidth::default()
            },
            emit_ssa: if self.emit_ssa { Some(package_build_path) } else { None },
            skip_underconstrained_check: !self.silence_warnings && self.skip_underconstrained_check,
            enable_brillig_constraints_check_lookback: self
                .enable_brillig_constraints_check_lookback,
            skip_brillig_constraints_check: !self.silence_warnings
                && self.skip_brillig_constraints_check,
            inliner_aggressiveness: self.inliner_aggressiveness,
            constant_folding_max_iter: self.constant_folding_max_iter,
            small_function_max_instruction: self.small_function_max_instructions,
            max_bytecode_increase_percent: self.max_bytecode_increase_percent,
            skip_passes: self.skip_ssa_pass.clone(),
        }
    }
}

pub fn parse_expression_width(input: &str) -> Result<ExpressionWidth, std::io::Error> {
    use std::io::{Error, ErrorKind};
    let width = input
        .parse::<usize>()
        .map_err(|err| Error::new(ErrorKind::InvalidInput, err.to_string()))?;

    match width {
        0 => Ok(ExpressionWidth::Unbounded),
        w if w >= MIN_EXPRESSION_WIDTH => Ok(ExpressionWidth::Bounded { width }),
        _ => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("has to be 0 or at least {MIN_EXPRESSION_WIDTH}"),
        )),
    }
}

impl CompileOptions {
    pub(crate) fn frontend_options(&self) -> FrontendOptions {
        FrontendOptions {
            debug_comptime_in_file: self.debug_comptime_in_file.as_deref(),
            pedantic_solving: self.pedantic_solving,
            enabled_unstable_features: &self.unstable_features,
            disable_required_unstable_features: self.no_unstable_features,
        }
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum CompileError {
    MonomorphizationError(MonomorphizationError),
    RuntimeError(RuntimeError),
}

impl From<MonomorphizationError> for CompileError {
    fn from(error: MonomorphizationError) -> Self {
        Self::MonomorphizationError(error)
    }
}

impl From<RuntimeError> for CompileError {
    fn from(error: RuntimeError) -> Self {
        Self::RuntimeError(error)
    }
}

impl From<CompileError> for CustomDiagnostic {
    fn from(error: CompileError) -> CustomDiagnostic {
        match error {
            CompileError::RuntimeError(err) => err.into(),
            CompileError::MonomorphizationError(err) => err.into(),
        }
    }
}

/// Helper type used to signify where only warnings are expected in file diagnostics
pub type Warnings = Vec<CustomDiagnostic>;

/// Helper type used to signify where errors or warnings are expected in file diagnostics
pub type ErrorsAndWarnings = Vec<CustomDiagnostic>;

/// Helper type for connecting a compilation artifact to the errors or warnings which were produced during compilation.
pub type CompilationResult<T> = Result<(T, Warnings), ErrorsAndWarnings>;

/// Helper method to return a file manager instance with the stdlib already added
///
/// TODO: This should become the canonical way to create a file manager and
/// TODO if we use a File manager trait, we can move file manager into this crate
/// TODO as a module
pub fn file_manager_with_stdlib(root: &Path) -> FileManager {
    let mut file_manager = FileManager::new(root);

    add_stdlib_source_to_file_manager(&mut file_manager);
    add_debug_source_to_file_manager(&mut file_manager);

    file_manager
}

/// Adds the source code for the stdlib into the file manager
fn add_stdlib_source_to_file_manager(file_manager: &mut FileManager) {
    // Add the stdlib contents to the file manager, since every package automatically has a dependency
    // on the stdlib. For other dependencies, we read the package.Dependencies file to add their file
    // contents to the file manager. However since the dependency on the stdlib is implicit, we need
    // to manually add it here.
    let stdlib_paths_with_source = stdlib_paths_with_source();
    for (path, source) in stdlib_paths_with_source {
        file_manager.add_file_with_source_canonical_path(Path::new(&path), source);
    }
}

/// Adds the source code of the debug crate needed to support instrumentation to
/// track variables values
fn add_debug_source_to_file_manager(file_manager: &mut FileManager) {
    // Adds the synthetic debug module for instrumentation into the file manager
    let path_to_debug_lib_file = Path::new(DEBUG_CRATE_NAME).join("lib.nr");
    file_manager
        .add_file_with_source_canonical_path(&path_to_debug_lib_file, build_debug_crate_file());
}

/// Adds the file from the file system at `Path` to the crate graph as a root file
///
/// Note: If the stdlib dependency has not been added yet, it's added. Otherwise
/// this method assumes the root crate is the stdlib (useful for running tests
/// in the stdlib, getting LSP stuff for the stdlib, etc.).
pub fn prepare_crate(context: &mut Context, file_name: &Path) -> CrateId {
    let path_to_std_lib_file = Path::new(STD_CRATE_NAME).join("lib.nr");
    let std_file_id = context.file_manager.name_to_id(path_to_std_lib_file);
    let std_crate_id = std_file_id.map(|std_file_id| context.crate_graph.add_stdlib(std_file_id));

    let root_file_id = context.file_manager.name_to_id(file_name.to_path_buf()).unwrap_or_else(|| panic!("files are expected to be added to the FileManager before reaching the compiler file_path: {file_name:?}"));

    if let Some(std_crate_id) = std_crate_id {
        let root_crate_id = context.crate_graph.add_crate_root(root_file_id);

        add_dep(context, root_crate_id, std_crate_id, STD_CRATE_NAME.parse().unwrap());

        root_crate_id
    } else {
        context.crate_graph.add_crate_root_and_stdlib(root_file_id)
    }
}

pub fn link_to_debug_crate(context: &mut Context, root_crate_id: CrateId) {
    let path_to_debug_lib_file = Path::new(DEBUG_CRATE_NAME).join("lib.nr");
    let debug_crate_id = prepare_dependency(context, &path_to_debug_lib_file);
    add_dep(context, root_crate_id, debug_crate_id, DEBUG_CRATE_NAME.parse().unwrap());
}

// Adds the file from the file system at `Path` to the crate graph
pub fn prepare_dependency(context: &mut Context, file_name: &Path) -> CrateId {
    let root_file_id = context
        .file_manager
        .name_to_id(file_name.to_path_buf())
        .unwrap_or_else(|| panic!("files are expected to be added to the FileManager before reaching the compiler file_path: {file_name:?}"));

    let crate_id = context.crate_graph.add_crate(root_file_id);

    // Every dependency has access to stdlib
    let std_crate_id = context.stdlib_crate_id();
    add_dep(context, crate_id, *std_crate_id, STD_CRATE_NAME.parse().unwrap());

    crate_id
}

/// Adds a edge in the crate graph for two crates
pub fn add_dep(
    context: &mut Context,
    this_crate: CrateId,
    depends_on: CrateId,
    crate_name: CrateName,
) {
    context
        .crate_graph
        .add_dep(this_crate, crate_name, depends_on)
        .expect("cyclic dependency triggered");
}

/// Run the lexing, parsing, name resolution, and type checking passes.
///
/// This returns a (possibly empty) vector of any warnings found on success.
/// On error, this returns a non-empty vector of warnings and error messages, with at least one error.
#[tracing::instrument(level = "trace", skip_all)]
pub fn check_crate(
    context: &mut Context,
    crate_id: CrateId,
    options: &CompileOptions,
) -> CompilationResult<()> {
    if options.disable_comptime_printing {
        context.disable_comptime_printing();
    }

    let diagnostics = CrateDefMap::collect_defs(crate_id, context, options.frontend_options());
    let crate_files = context.crate_files(&crate_id);
    let warnings_and_errors: Vec<CustomDiagnostic> = diagnostics
        .iter()
        .map(CustomDiagnostic::from)
        .filter(|diagnostic| {
            // We filter out any warnings if they're going to be ignored later on to free up memory.
            !options.silence_warnings || diagnostic.kind != DiagnosticKind::Warning
        })
        .filter(|error| {
            // Only keep warnings from the crate we are checking
            if error.is_warning() { crate_files.contains(&error.file) } else { true }
        })
        .collect();

    if has_errors(&warnings_and_errors, options.deny_warnings) {
        Err(warnings_and_errors)
    } else {
        Ok(((), warnings_and_errors))
    }
}

pub fn compute_function_abi(
    context: &Context,
    crate_id: &CrateId,
) -> Option<(Vec<AbiParameter>, Option<AbiType>)> {
    let main_function = context.get_main_function(crate_id)?;

    Some(abi_gen::compute_function_abi(context, &main_function))
}

/// Run the frontend to check the crate for errors then compile the main function if there were none
///
/// On success this returns the compiled program alongside any warnings that were found.
/// On error this returns the non-empty list of warnings and errors.
///
/// See [compile_no_check] for further information about the use of `cached_program`.
pub fn compile_main(
    context: &mut Context,
    crate_id: CrateId,
    options: &CompileOptions,
    cached_program: Option<CompiledProgram>,
) -> CompilationResult<CompiledProgram> {
    let (_, mut warnings) = check_crate(context, crate_id, options)?;

    let main = context.get_main_function(&crate_id).ok_or_else(|| {
        // TODO(#2155): This error might be a better to exist in Nargo
        let err = CustomDiagnostic::from_message(
            "cannot compile crate into a program as it does not contain a `main` function",
            FileId::default(),
        );
        vec![err]
    })?;

    let compiled_program =
        compile_no_check(context, options, main, cached_program, options.force_compile)
            .map_err(|error| vec![CustomDiagnostic::from(error)])?;

    let compilation_warnings = vecmap(compiled_program.warnings.clone(), CustomDiagnostic::from);
    if options.deny_warnings && !compilation_warnings.is_empty() {
        return Err(compilation_warnings);
    }
    if !options.silence_warnings {
        warnings.extend(compilation_warnings);
    }

    if options.print_acir {
        noirc_errors::println_to_stdout!("Compiled ACIR for main (non-transformed):");
        noirc_errors::println_to_stdout!("{}", compiled_program.program);
    }

    Ok((compiled_program, warnings))
}

/// Run the frontend to check the crate for errors then compile all contracts if there were none
pub fn compile_contract(
    context: &mut Context,
    crate_id: CrateId,
    options: &CompileOptions,
) -> CompilationResult<CompiledContract> {
    let (_, warnings) = check_crate(context, crate_id, options)?;

    let def_map = context.def_map(&crate_id).expect("The local crate should be analyzed already");
    let mut contracts = def_map.get_all_contracts();

    let Some((module_id, name)) = contracts.next() else {
        let err = CustomDiagnostic::from_message(
            "cannot compile crate into a contract as it does not contain any contracts",
            FileId::default(),
        );
        return Err(vec![err]);
    };

    if contracts.next().is_some() {
        let err = CustomDiagnostic::from_message(
            "Packages are limited to a single contract",
            FileId::default(),
        );
        return Err(vec![err]);
    }
    drop(contracts);

    let module_id = ModuleId { krate: crate_id, local_id: module_id };
    let contract = read_contract(context, module_id, name);

    let mut errors = warnings;

    let compiled_contract = match compile_contract_inner(context, contract, options) {
        Ok(contract) => contract,
        Err(mut more_errors) => {
            errors.append(&mut more_errors);
            return Err(errors);
        }
    };

    if has_errors(&errors, options.deny_warnings) {
        Err(errors)
    } else {
        if options.print_acir {
            for contract_function in &compiled_contract.functions {
                if let Some(ref name) = options.show_contract_fn {
                    if name != &contract_function.name {
                        continue;
                    }
                }
                println!(
                    "Compiled ACIR for {}::{} (non-transformed):",
                    compiled_contract.name, contract_function.name
                );
                println!("{}", contract_function.bytecode);
            }
        }
        // errors here is either empty or contains only warnings
        Ok((compiled_contract, errors))
    }
}

/// Return a Vec of all `contract` declarations in the source code and the functions they contain
fn read_contract(context: &Context, module_id: ModuleId, name: String) -> Contract {
    let module = context.module(module_id);

    let functions: Vec<ContractFunctionMeta> = module
        .value_definitions()
        .filter_map(|id| {
            id.as_function().map(|function_id| {
                let attrs = context.def_interner.function_attributes(&function_id);
                let is_entry_point = attrs.is_contract_entry_point();
                ContractFunctionMeta { function_id, is_entry_point }
            })
        })
        .collect();

    let mut outputs = ContractOutputs { structs: HashMap::new(), globals: HashMap::new() };

    context.def_interner.get_all_globals().iter().for_each(|global_info| {
        context.def_interner.global_attributes(&global_info.id).iter().for_each(|attr| {
            if let SecondaryAttributeKind::Abi(tag) = &attr.kind {
                if let Some(tagged) = outputs.globals.get_mut(tag) {
                    tagged.push(global_info.id);
                } else {
                    outputs.globals.insert(tag.to_string(), vec![global_info.id]);
                }
            }
        });
    });

    module.type_definitions().for_each(|id| {
        if let ModuleDefId::TypeId(struct_id) = id {
            context.def_interner.type_attributes(&struct_id).iter().for_each(|attr| {
                if let SecondaryAttributeKind::Abi(tag) = &attr.kind {
                    if let Some(tagged) = outputs.structs.get_mut(tag) {
                        tagged.push(struct_id);
                    } else {
                        outputs.structs.insert(tag.to_string(), vec![struct_id]);
                    }
                }
            });
        }
    });

    Contract { name, functions, outputs }
}

/// True if there are (non-warning) errors present and we should halt compilation
fn has_errors(errors: &[CustomDiagnostic], deny_warnings: bool) -> bool {
    if deny_warnings { !errors.is_empty() } else { errors.iter().any(|error| error.is_error()) }
}

/// Compile all of the functions associated with a Noir contract.
fn compile_contract_inner(
    context: &mut Context,
    contract: Contract,
    options: &CompileOptions,
) -> Result<CompiledContract, ErrorsAndWarnings> {
    let mut functions = Vec::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for contract_function in &contract.functions {
        let function_id = contract_function.function_id;
        let is_entry_point = contract_function.is_entry_point;

        let name = context.function_name(&function_id).to_owned();

        // We assume that functions have already been type checked.
        // This is the exact same assumption that compile_no_check makes.
        // If it is not an entry-point point, we can then just skip the
        // compilation step. It will also not be added to the ABI.
        if !is_entry_point {
            continue;
        }

        let mut options = options.clone();
        if name == "public_dispatch" {
            options.inliner_aggressiveness = 0;
        }

        if let Some(ref name_filter) = options.show_contract_fn {
            let show = name == *name_filter;
            options.show_ssa &= show;
            if !show {
                options.show_ssa_pass.clear();
            }
        };

        let function = match compile_no_check(context, &options, function_id, None, true) {
            Ok(function) => function,
            Err(new_error) => {
                errors.push(new_error.into());
                continue;
            }
        };
        warnings.extend(function.warnings);
        let modifiers = context.def_interner.function_modifiers(&function_id);

        let custom_attributes = modifiers
            .attributes
            .secondary
            .iter()
            .filter_map(|attr| match &attr.kind {
                SecondaryAttributeKind::Tag(contents) => Some(contents.clone()),
                SecondaryAttributeKind::Meta(meta_attribute) => {
                    context.def_interner.get_meta_attribute_name(meta_attribute)
                }
                _ => None,
            })
            .collect();

        functions.push(ContractFunction {
            name,
            hash: function.hash,
            custom_attributes,
            abi: function.abi,
            bytecode: function.program,
            debug: function.debug,
            is_unconstrained: modifiers.is_unconstrained,
            expression_width: options.expression_width.unwrap_or(DEFAULT_EXPRESSION_WIDTH),
        });
    }

    if errors.is_empty() {
        let debug_infos: Vec<_> =
            functions.iter().flat_map(|function| function.debug.clone()).collect();
        let file_map = filter_relevant_files(&debug_infos, &context.file_manager);

        let out_structs = contract
            .outputs
            .structs
            .into_iter()
            .map(|(tag, structs)| {
                let structs = structs
                    .into_iter()
                    .map(|struct_id| {
                        let typ = context.def_interner.get_type(struct_id);
                        let typ = typ.borrow();
                        let fields =
                            vecmap(typ.get_fields(&[]).unwrap_or_default(), |(name, typ, _)| {
                                (name, abi_type_from_hir_type(context, &typ))
                            });
                        let path =
                            context.fully_qualified_struct_path(context.root_crate_id(), typ.id);
                        AbiType::Struct { path, fields }
                    })
                    .collect();
                (tag.to_string(), structs)
            })
            .collect();

        let out_globals = contract
            .outputs
            .globals
            .iter()
            .map(|(tag, globals)| {
                let globals: Vec<AbiValue> = globals
                    .iter()
                    .map(|global_id| {
                        let let_statement =
                            context.def_interner.get_global_let_statement(*global_id).unwrap();
                        let hir_expression =
                            context.def_interner.expression(&let_statement.expression);
                        value_from_hir_expression(context, hir_expression)
                    })
                    .collect();
                (tag.to_string(), globals)
            })
            .collect();

        Ok(CompiledContract {
            name: contract.name,
            functions,
            outputs: CompiledContractOutputs { structs: out_structs, globals: out_globals },
            file_map,
            noir_version: NOIR_ARTIFACT_VERSION_STRING.to_string(),
            warnings,
        })
    } else {
        Err(errors)
    }
}

/// Default expression width used for Noir compilation.
/// The ACVM native type `ExpressionWidth` has its own default which should always be unbounded,
/// while we can sometimes expect the compilation target width to change.
/// Thus, we set it separately here rather than trying to alter the default derivation of the type.
pub const DEFAULT_EXPRESSION_WIDTH: ExpressionWidth = ExpressionWidth::Bounded { width: 4 };

/// Compile the current crate using `main_function` as the entrypoint.
///
/// This function assumes [`check_crate`] is called beforehand.
///
/// If the program is not returned from cache, it is backend-agnostic and must go through a transformation
/// pass before usage in proof generation; if it's returned from cache these transformations might have
/// already been applied.
///
/// The transformations are _not_ covered by the check that decides whether we can use the cached artifact.
/// That comparison is based on on [CompiledProgram::hash] which is a persisted version of the hash of the input
/// [`ast::Program`][noirc_frontend::monomorphization::ast::Program], whereas the output [`circuit::Program`][acvm::acir::circuit::Program]
/// contains the final optimized ACIR opcodes, including the transformation done after this compilation.
#[tracing::instrument(level = "trace", skip_all, fields(function_name = context.function_name(&main_function)))]
#[allow(clippy::result_large_err)]
pub fn compile_no_check(
    context: &mut Context,
    options: &CompileOptions,
    main_function: FuncId,
    cached_program: Option<CompiledProgram>,
    force_compile: bool,
) -> Result<CompiledProgram, CompileError> {
    let force_unconstrained = options.force_brillig || options.minimal_ssa;

    let program = if options.instrument_debug {
        monomorphize_debug(
            main_function,
            &mut context.def_interner,
            &context.debug_instrumenter,
            force_unconstrained,
        )?
    } else {
        monomorphize(main_function, &mut context.def_interner, force_unconstrained)?
    };

    if options.show_monomorphized {
        println!("{program}");
    }

    // If user has specified that they want to see intermediate steps printed then we should
    // force compilation even if the program hasn't changed.
    let force_compile = force_compile
        || options.print_acir
        || options.show_brillig
        || options.force_brillig
        || options.count_array_copies
        || options.show_ssa
        || !options.show_ssa_pass.is_empty()
        || options.emit_ssa
        || options.minimal_ssa;

    // Hash the AST program, which is going to be used to fingerprint the compilation artifact.
    let hash = rustc_hash::FxBuildHasher.hash_one(&program);

    if let Some(cached_program) = cached_program {
        if !force_compile && cached_program.hash == hash {
            info!("Program matches existing artifact, returning early");
            return Ok(cached_program);
        }
    }

    let return_visibility = program.return_visibility();
    let ssa_evaluator_options = options.as_ssa_options(context.package_build_path.clone());

    let SsaProgramArtifact { program, debug, warnings, error_types, .. } = if options.minimal_ssa {
        create_program_with_minimal_passes(program, &ssa_evaluator_options, &context.file_manager)?
    } else {
        create_program(
            program,
            &ssa_evaluator_options,
            if options.with_ssa_locations { Some(&context.file_manager) } else { None },
        )?
    };

    let abi = gen_abi(context, &main_function, return_visibility, error_types);
    let file_map = filter_relevant_files(&debug, &context.file_manager);

    Ok(CompiledProgram {
        hash,
        program,
        debug,
        abi,
        file_map,
        noir_version: NOIR_ARTIFACT_VERSION_STRING.to_string(),
        warnings,
        expression_width: options.expression_width.unwrap_or(DEFAULT_EXPRESSION_WIDTH),
    })
}

/// Specifies a contract function and extra metadata that
/// one can use when processing a contract function.
///
/// One of these is whether the contract function is an entry point.
/// The caller should only type-check these functions and not attempt
/// to create a circuit for them.
struct ContractFunctionMeta {
    function_id: FuncId,
    /// Indicates whether the function is an entry point
    is_entry_point: bool,
}

struct ContractOutputs {
    structs: HashMap<String, Vec<TypeId>>,
    globals: HashMap<String, Vec<GlobalId>>,
}

/// A 'contract' in Noir source code with a given name, functions and events.
/// This is not an AST node, it is just a convenient form to return for CrateDefMap::get_all_contracts.
struct Contract {
    /// To keep `name` semi-unique, it is prefixed with the names of parent modules via CrateDefMap::get_module_path
    name: String,
    functions: Vec<ContractFunctionMeta>,
    outputs: ContractOutputs,
}
