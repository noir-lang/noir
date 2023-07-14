#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

use clap::Args;
use fm::FileId;
use noirc_abi::FunctionSignature;
use noirc_errors::{CustomDiagnostic, FileDiagnostic};
use noirc_evaluator::{create_circuit, ssa_refactor::experimental_create_circuit};
use noirc_frontend::graph::{CrateId, CrateName, CrateType, LOCAL_CRATE};
use noirc_frontend::hir::def_map::{Contract, CrateDefMap};
use noirc_frontend::hir::Context;
use noirc_frontend::monomorphization::monomorphize;
use noirc_frontend::node_interner::FuncId;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

mod contract;
mod program;

pub use contract::{CompiledContract, ContractFunction, ContractFunctionType};
pub use program::CompiledProgram;

#[derive(Args, Clone, Debug, Serialize, Deserialize)]
pub struct CompileOptions {
    /// Emit debug information for the intermediate SSA IR
    #[arg(short, long)]
    pub show_ssa: bool,

    /// Display the ACIR for compiled circuit
    #[arg(long)]
    pub print_acir: bool,

    /// Treat all warnings as errors
    #[arg(short, long)]
    pub deny_warnings: bool,

    /// Display output of `println` statements
    #[arg(long)]
    pub show_output: bool,

    /// Compile and optimize using the new experimental SSA pass
    #[arg(long)]
    pub experimental_ssa: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            show_ssa: false,
            print_acir: false,
            deny_warnings: false,
            show_output: true,
            experimental_ssa: false,
        }
    }
}

/// Helper type used to signify where only warnings are expected in file diagnostics
pub type Warnings = Vec<FileDiagnostic>;

/// Helper type used to signify where errors or warnings are expected in file diagnostics
pub type ErrorsAndWarnings = Vec<FileDiagnostic>;

// This is here for backwards compatibility
// with the restricted version which only uses one file
pub fn compile_file(
    context: &mut Context,
    root_file: PathBuf,
) -> Result<(CompiledProgram, Warnings), ErrorsAndWarnings> {
    create_local_crate(context, root_file, CrateType::Binary);
    compile_main(context, &CompileOptions::default())
}

/// Adds the File with the local crate root to the file system
/// and adds the local crate to the graph
/// XXX: This may pose a problem with workspaces, where you can change the local crate and where
/// we have multiple binaries in one workspace
/// A Fix would be for the driver instance to store the local crate id.
// Granted that this is the only place which relies on the local crate being first
pub fn create_local_crate<P: AsRef<Path>>(
    context: &mut Context,
    root_file: P,
    crate_type: CrateType,
) -> CrateId {
    let dir_path = root_file.as_ref().to_path_buf();
    let root_file_id = context.file_manager.add_file(&dir_path).unwrap();

    let crate_id = context.crate_graph.add_crate_root(crate_type, root_file_id);

    assert!(crate_id == LOCAL_CRATE);

    LOCAL_CRATE
}

/// Creates a Non Local Crate. A Non Local Crate is any crate which is the not the crate that
/// the compiler is compiling.
pub fn create_non_local_crate<P: AsRef<Path>>(
    context: &mut Context,
    root_file: P,
    crate_type: CrateType,
) -> CrateId {
    let dir_path = root_file.as_ref().to_path_buf();
    let root_file_id = context.file_manager.add_file(&dir_path).unwrap();

    // The first crate is always the local crate
    assert!(context.crate_graph.number_of_crates() != 0);

    // You can add any crate type to the crate graph
    // but you cannot depend on Binaries
    context.crate_graph.add_crate_root(crate_type, root_file_id)
}

/// Adds a edge in the crate graph for two crates
pub fn add_dep(context: &mut Context, this_crate: CrateId, depends_on: CrateId, crate_name: &str) {
    let crate_name = CrateName::new(crate_name)
        .expect("crate name contains blacklisted characters, please remove");

    // Cannot depend on a binary
    if context.crate_graph.crate_type(depends_on) == CrateType::Binary {
        panic!("crates cannot depend on binaries. {crate_name:?} is a binary crate")
    }

    context
        .crate_graph
        .add_dep(this_crate, crate_name, depends_on)
        .expect("cyclic dependency triggered");
}

/// Propagates a given dependency to every other crate.
pub fn propagate_dep(
    context: &mut Context,
    dep_to_propagate: CrateId,
    dep_to_propagate_name: &CrateName,
) {
    let crate_ids: Vec<_> =
        context.crate_graph.iter_keys().filter(|crate_id| *crate_id != dep_to_propagate).collect();

    for crate_id in crate_ids {
        context
            .crate_graph
            .add_dep(crate_id, dep_to_propagate_name.clone(), dep_to_propagate)
            .expect("ice: cyclic error triggered with std library");
    }
}

/// Run the lexing, parsing, name resolution, and type checking passes.
///
/// This returns a (possibly empty) vector of any warnings found on success.
/// On error, this returns a non-empty vector of warnings and error messages, with at least one error.
pub fn check_crate(
    context: &mut Context,
    deny_warnings: bool,
    enable_slices: bool,
) -> Result<Warnings, ErrorsAndWarnings> {
    // Add the stdlib before we check the crate
    // TODO: This should actually be done when constructing the driver and then propagated to each dependency when added;
    // however, the `create_non_local_crate` panics if you add the stdlib as the first crate in the graph and other
    // parts of the code expect the `0` FileID to be the crate root. See also #1681
    let std_crate_name = "std";
    let path_to_std_lib_file = PathBuf::from(std_crate_name).join("lib.nr");
    let std_crate = create_non_local_crate(context, path_to_std_lib_file, CrateType::Library);
    propagate_dep(context, std_crate, &CrateName::new(std_crate_name).unwrap());

    context.def_interner.enable_slices = enable_slices;

    let mut errors = vec![];
    match context.crate_graph.crate_type(LOCAL_CRATE) {
        CrateType::Workspace => {
            let keys: Vec<_> = context.crate_graph.iter_keys().collect(); // avoid borrow checker
            for crate_id in keys {
                CrateDefMap::collect_defs(crate_id, context, &mut errors);
            }
        }
        _ => CrateDefMap::collect_defs(LOCAL_CRATE, context, &mut errors),
    }

    if has_errors(&errors, deny_warnings) {
        Err(errors)
    } else {
        Ok(errors)
    }
}

pub fn compute_function_signature(context: &Context) -> Option<FunctionSignature> {
    let main_function = context.get_main_function(&LOCAL_CRATE)?;

    let func_meta = context.def_interner.function_meta(&main_function);

    Some(func_meta.into_function_signature(&context.def_interner))
}

/// Run the frontend to check the crate for errors then compile the main function if there were none
///
/// On success this returns the compiled program alongside any warnings that were found.
/// On error this returns the non-empty list of warnings and errors.
pub fn compile_main(
    context: &mut Context,
    options: &CompileOptions,
) -> Result<(CompiledProgram, Warnings), ErrorsAndWarnings> {
    let warnings = check_crate(context, options.deny_warnings, options.experimental_ssa)?;

    let main = match context.get_main_function(&LOCAL_CRATE) {
        Some(m) => m,
        None => {
            let err = FileDiagnostic {
                    file_id: FileId::default(),
                    diagnostic: CustomDiagnostic::from_message("cannot compile crate into a program as the local crate is not a binary. For libraries, please use the check command")
                };
            return Err(vec![err]);
        }
    };

    let compiled_program = compile_no_check(context, options, main)?;

    if options.print_acir {
        println!("Compiled ACIR for main:");
        println!("{}", compiled_program.circuit);
    }

    Ok((compiled_program, warnings))
}

/// Run the frontend to check the crate for errors then compile all contracts if there were none
pub fn compile_contracts(
    context: &mut Context,
    options: &CompileOptions,
) -> Result<(Vec<CompiledContract>, Warnings), ErrorsAndWarnings> {
    let warnings = check_crate(context, options.deny_warnings, options.experimental_ssa)?;

    let contracts = context.get_all_contracts(&LOCAL_CRATE);
    let mut compiled_contracts = vec![];
    let mut errors = warnings;

    for contract in contracts {
        match compile_contract(context, contract, options) {
            Ok(contract) => compiled_contracts.push(contract),
            Err(mut more_errors) => errors.append(&mut more_errors),
        }
    }

    if has_errors(&errors, options.deny_warnings) {
        Err(errors)
    } else {
        if options.print_acir {
            for compiled_contract in &compiled_contracts {
                for contract_function in &compiled_contract.functions {
                    println!(
                        "Compiled ACIR for {}::{}:",
                        compiled_contract.name, contract_function.name
                    );
                    println!("{}", contract_function.bytecode);
                }
            }
        }
        // errors here is either empty or contains only warnings
        Ok((compiled_contracts, errors))
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
fn compile_contract(
    context: &Context,
    contract: Contract,
    options: &CompileOptions,
) -> Result<CompiledContract, Vec<FileDiagnostic>> {
    let mut functions = Vec::new();
    let mut errs = Vec::new();
    for function_id in &contract.functions {
        let name = context.function_name(function_id).to_owned();
        let function = match compile_no_check(context, options, *function_id) {
            Ok(function) => function,
            Err(err) => {
                errs.push(err);
                continue;
            }
        };
        let func_meta = context.def_interner.function_meta(function_id);
        let func_type = func_meta
            .contract_function_type
            .expect("Expected contract function to have a contract visibility");

        let function_type = ContractFunctionType::new(func_type, func_meta.is_unconstrained);

        functions.push(ContractFunction {
            name,
            function_type,
            is_internal: func_meta.is_internal.unwrap_or(false),
            abi: function.abi,
            bytecode: function.circuit,
        });
    }

    if errs.is_empty() {
        Ok(CompiledContract { name: contract.name, functions })
    } else {
        Err(errs)
    }
}

/// Compile the current crate. Assumes self.check_crate is called beforehand!
///
/// This function also assumes all errors in experimental_create_circuit and create_circuit
/// are not warnings.
#[allow(deprecated)]
pub fn compile_no_check(
    context: &Context,
    options: &CompileOptions,
    main_function: FuncId,
) -> Result<CompiledProgram, FileDiagnostic> {
    let program = monomorphize(main_function, &context.def_interner);

    let (circuit, debug, abi) = if options.experimental_ssa {
        experimental_create_circuit(program, options.show_ssa, options.show_output)?
    } else {
        create_circuit(program, options.show_ssa, options.show_output)?
    };

    Ok(CompiledProgram { circuit, debug, abi })
}
