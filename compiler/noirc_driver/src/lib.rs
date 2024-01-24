#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

use acvm::ExpressionWidth;
use clap::Args;
use fm::{FileId, FileManager};
use iter_extended::vecmap;
use noirc_abi::{AbiParameter, AbiType, ContractEvent};
use noirc_errors::{CustomDiagnostic, FileDiagnostic};
use noirc_evaluator::create_circuit;
use noirc_evaluator::errors::RuntimeError;
use noirc_frontend::graph::{CrateId, CrateName};
use noirc_frontend::hir::def_map::{Contract, CrateDefMap};
use noirc_frontend::hir::Context;
use noirc_frontend::macros_api::MacroProcessor;
use noirc_frontend::monomorphization::monomorphize;
use noirc_frontend::node_interner::FuncId;
use std::path::Path;
use tracing::info;

mod abi_gen;
mod contract;
mod debug;
mod program;
mod stdlib;

use debug::filter_relevant_files;

pub use contract::{CompiledContract, ContractFunction, ContractFunctionType};
pub use debug::DebugFile;
pub use program::CompiledProgram;

const STD_CRATE_NAME: &str = "std";

pub const GIT_COMMIT: &str = env!("GIT_COMMIT");
pub const GIT_DIRTY: &str = env!("GIT_DIRTY");
pub const NOIRC_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Version string that gets placed in artifacts that Noir builds. This is semver compatible.
/// Note: You can't directly use the value of a constant produced with env! inside a concat! macro.
pub const NOIR_ARTIFACT_VERSION_STRING: &str =
    concat!(env!("CARGO_PKG_VERSION"), "+", env!("GIT_COMMIT"));

#[derive(Args, Clone, Debug, Default)]
pub struct CompileOptions {
    /// Override the expression width requested by the backend.
    #[arg(long, value_parser = parse_expression_width)]
    pub expression_width: Option<ExpressionWidth>,

    /// Force a full recompilation.
    #[arg(long = "force")]
    pub force_compile: bool,

    /// Emit debug information for the intermediate SSA IR
    #[arg(long, hide = true)]
    pub show_ssa: bool,

    #[arg(long, hide = true)]
    pub show_brillig: bool,

    /// Display the ACIR for compiled circuit
    #[arg(long)]
    pub print_acir: bool,

    /// Treat all warnings as errors
    #[arg(long, conflicts_with = "silence_warnings")]
    pub deny_warnings: bool,

    /// Suppress warnings
    #[arg(long, conflicts_with = "deny_warnings")]
    pub silence_warnings: bool,

    /// Output ACIR gzipped bytecode instead of the JSON artefact
    #[arg(long, hide = true)]
    pub only_acir: bool,

    /// Disables the builtin macros being used in the compiler
    #[arg(long, hide = true)]
    pub disable_macros: bool,

    /// Outputs the monomorphized IR to stdout for debugging
    #[arg(long, hide = true)]
    pub show_monomorphized: bool,
}

fn parse_expression_width(input: &str) -> Result<ExpressionWidth, std::io::Error> {
    use std::io::{Error, ErrorKind};

    let width = input
        .parse::<usize>()
        .map_err(|err| Error::new(ErrorKind::InvalidInput, err.to_string()))?;

    Ok(ExpressionWidth::from(width))
}

/// Helper type used to signify where only warnings are expected in file diagnostics
pub type Warnings = Vec<FileDiagnostic>;

/// Helper type used to signify where errors or warnings are expected in file diagnostics
pub type ErrorsAndWarnings = Vec<FileDiagnostic>;

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

    file_manager
}

/// Adds the source code for the stdlib into the file manager
fn add_stdlib_source_to_file_manager(file_manager: &mut FileManager) {
    // Add the stdlib contents to the file manager, since every package automatically has a dependency
    // on the stdlib. For other dependencies, we read the package.Dependencies file to add their file
    // contents to the file manager. However since the dependency on the stdlib is implicit, we need
    // to manually add it here.
    let stdlib_paths_with_source = stdlib::stdlib_paths_with_source();
    for (path, source) in stdlib_paths_with_source {
        file_manager.add_file_with_source_canonical_path(Path::new(&path), source);
    }
}

/// Adds the file from the file system at `Path` to the crate graph as a root file
///
/// Note: This methods adds the stdlib as a dependency to the crate.
/// This assumes that the stdlib has already been added to the file manager.
pub fn prepare_crate(context: &mut Context, file_name: &Path) -> CrateId {
    let path_to_std_lib_file = Path::new(STD_CRATE_NAME).join("lib.nr");
    let std_file_id = context
        .file_manager
        .name_to_id(path_to_std_lib_file)
        .expect("stdlib file id is expected to be present");
    let std_crate_id = context.crate_graph.add_stdlib(std_file_id);

    let root_file_id = context.file_manager.name_to_id(file_name.to_path_buf()).unwrap_or_else(|| panic!("files are expected to be added to the FileManager before reaching the compiler file_path: {file_name:?}"));

    let root_crate_id = context.crate_graph.add_crate_root(root_file_id);

    add_dep(context, root_crate_id, std_crate_id, STD_CRATE_NAME.parse().unwrap());

    root_crate_id
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
#[tracing::instrument(level = "trace", skip(context))]
pub fn check_crate(
    context: &mut Context,
    crate_id: CrateId,
    deny_warnings: bool,
    disable_macros: bool,
) -> CompilationResult<()> {
    let macros: Vec<&dyn MacroProcessor> = if disable_macros {
        vec![]
    } else {
        vec![&aztec_macros::AztecMacro as &dyn MacroProcessor]
    };

    let mut errors = vec![];
    let diagnostics = CrateDefMap::collect_defs(crate_id, context, macros);
    errors.extend(diagnostics.into_iter().map(|(error, file_id)| {
        let diagnostic: CustomDiagnostic = error.into();
        diagnostic.in_file(file_id)
    }));

    if has_errors(&errors, deny_warnings) {
        Err(errors)
    } else {
        Ok(((), errors))
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
pub fn compile_main(
    context: &mut Context,
    crate_id: CrateId,
    options: &CompileOptions,
    cached_program: Option<CompiledProgram>,
) -> CompilationResult<CompiledProgram> {
    let (_, mut warnings) =
        check_crate(context, crate_id, options.deny_warnings, options.disable_macros)?;

    let main = context.get_main_function(&crate_id).ok_or_else(|| {
        // TODO(#2155): This error might be a better to exist in Nargo
        let err = CustomDiagnostic::from_message(
            "cannot compile crate into a program as it does not contain a `main` function",
        )
        .in_file(FileId::default());
        vec![err]
    })?;

    let compiled_program =
        compile_no_check(context, options, main, cached_program, options.force_compile)
            .map_err(FileDiagnostic::from)?;
    let compilation_warnings = vecmap(compiled_program.warnings.clone(), FileDiagnostic::from);
    if options.deny_warnings && !compilation_warnings.is_empty() {
        return Err(compilation_warnings);
    }
    warnings.extend(compilation_warnings);

    if options.print_acir {
        println!("Compiled ACIR for main (unoptimized):");
        println!("{}", compiled_program.circuit);
    }

    Ok((compiled_program, warnings))
}

/// Run the frontend to check the crate for errors then compile all contracts if there were none
pub fn compile_contract(
    context: &mut Context,
    crate_id: CrateId,
    options: &CompileOptions,
) -> CompilationResult<CompiledContract> {
    let (_, warnings) =
        check_crate(context, crate_id, options.deny_warnings, options.disable_macros)?;

    // TODO: We probably want to error if contracts is empty
    let contracts = context.get_all_contracts(&crate_id);

    let mut compiled_contracts = vec![];
    let mut errors = warnings;

    if contracts.len() > 1 {
        let err = CustomDiagnostic::from_message("Packages are limited to a single contract")
            .in_file(FileId::default());
        return Err(vec![err]);
    } else if contracts.is_empty() {
        let err = CustomDiagnostic::from_message(
            "cannot compile crate into a contract as it does not contain any contracts",
        )
        .in_file(FileId::default());
        return Err(vec![err]);
    };

    for contract in contracts {
        match compile_contract_inner(context, contract, options) {
            Ok(contract) => compiled_contracts.push(contract),
            Err(mut more_errors) => errors.append(&mut more_errors),
        }
    }

    if has_errors(&errors, options.deny_warnings) {
        Err(errors)
    } else {
        assert_eq!(compiled_contracts.len(), 1);
        let compiled_contract = compiled_contracts.remove(0);

        if options.print_acir {
            for contract_function in &compiled_contract.functions {
                println!(
                    "Compiled ACIR for {}::{} (unoptimized):",
                    compiled_contract.name, contract_function.name
                );
                println!("{}", contract_function.bytecode);
            }
        }
        // errors here is either empty or contains only warnings
        Ok((compiled_contract, errors))
    }
}

/// True if there are (non-warning) errors present and we should halt compilation
fn has_errors(errors: &[FileDiagnostic], deny_warnings: bool) -> bool {
    if deny_warnings {
        !errors.is_empty()
    } else {
        errors.iter().any(|error| error.diagnostic.is_error())
    }
}

/// Compile all of the functions associated with a Noir contract.
fn compile_contract_inner(
    context: &Context,
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

        let function = match compile_no_check(context, options, function_id, None, true) {
            Ok(function) => function,
            Err(new_error) => {
                errors.push(FileDiagnostic::from(new_error));
                continue;
            }
        };
        warnings.extend(function.warnings);
        let modifiers = context.def_interner.function_modifiers(&function_id);
        let func_type = modifiers
            .contract_function_type
            .expect("Expected contract function to have a contract visibility");

        let function_type = ContractFunctionType::new(func_type, modifiers.is_unconstrained);

        functions.push(ContractFunction {
            name,
            function_type,
            is_internal: modifiers.is_internal.unwrap_or(false),
            abi: function.abi,
            bytecode: function.circuit,
            debug: function.debug,
        });
    }

    if errors.is_empty() {
        let debug_infos: Vec<_> = functions.iter().map(|function| function.debug.clone()).collect();
        let file_map = filter_relevant_files(&debug_infos, &context.file_manager);

        Ok(CompiledContract {
            name: contract.name,
            events: contract
                .events
                .iter()
                .map(|event_id| {
                    let typ = context.def_interner.get_struct(*event_id);
                    let typ = typ.borrow();
                    ContractEvent::from_struct_type(context, &typ)
                })
                .collect(),
            functions,
            file_map,
            noir_version: NOIR_ARTIFACT_VERSION_STRING.to_string(),
            warnings,
        })
    } else {
        Err(errors)
    }
}

/// Compile the current crate using `main_function` as the entrypoint.
///
/// This function assumes [`check_crate`] is called beforehand.
#[tracing::instrument(level = "trace", skip_all, fields(function_name = context.function_name(&main_function)))]
pub fn compile_no_check(
    context: &Context,
    options: &CompileOptions,
    main_function: FuncId,
    cached_program: Option<CompiledProgram>,
    force_compile: bool,
) -> Result<CompiledProgram, RuntimeError> {
    let program = monomorphize(main_function, &context.def_interner);

    let hash = fxhash::hash64(&program);
    let hashes_match = cached_program.as_ref().map_or(false, |program| program.hash == hash);
    if options.show_monomorphized {
        println!("{program}");
    }

    // If user has specified that they want to see intermediate steps printed then we should
    // force compilation even if the program hasn't changed.
    let force_compile =
        force_compile || options.print_acir || options.show_brillig || options.show_ssa;

    if !force_compile && hashes_match {
        info!("Program matches existing artifact, returning early");
        return Ok(cached_program.expect("cache must exist for hashes to match"));
    }
    let visibility = program.return_visibility;
    let (circuit, debug, input_witnesses, return_witnesses, warnings) =
        create_circuit(program, options.show_ssa, options.show_brillig)?;

    let abi =
        abi_gen::gen_abi(context, &main_function, input_witnesses, return_witnesses, visibility);
    let file_map = filter_relevant_files(&[debug.clone()], &context.file_manager);

    Ok(CompiledProgram {
        hash,
        circuit,
        debug,
        abi,
        file_map,
        noir_version: NOIR_ARTIFACT_VERSION_STRING.to_string(),
        warnings,
    })
}
