#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

use clap::Args;
use fm::FileId;
use noirc_abi::{AbiParameter, AbiType};
use noirc_errors::{CustomDiagnostic, FileDiagnostic};
use noirc_evaluator::{create_circuit, into_abi_params};
use noirc_frontend::graph::{CrateId, CrateName};
use noirc_frontend::hir::def_map::{Contract, CrateDefMap};
use noirc_frontend::hir::visibility::FunctionVisibility;
use noirc_frontend::hir::Context;
use noirc_frontend::monomorphization::monomorphize;
use noirc_frontend::node_interner::FuncId;
use serde::{Deserialize, Serialize};
use std::path::Path;

mod contract;
mod program;

pub use contract::{CompiledContract, ContractFunction, ContractFunctionType};
pub use program::CompiledProgram;

const STD_CRATE_NAME: &str = "std";

#[derive(Args, Clone, Debug, Default, Serialize, Deserialize)]
pub struct CompileOptions {
    /// Emit debug information for the intermediate SSA IR
    #[arg(long, hide = true)]
    pub show_ssa: bool,

    #[arg(long, hide = true)]
    pub show_brillig: bool,

    /// Display the ACIR for compiled circuit
    #[arg(long)]
    pub print_acir: bool,

    /// Treat all warnings as errors
    #[arg(long)]
    pub deny_warnings: bool,
}

/// Helper type used to signify where only warnings are expected in file diagnostics
pub type Warnings = Vec<FileDiagnostic>;

/// Helper type used to signify where errors or warnings are expected in file diagnostics
pub type ErrorsAndWarnings = Vec<FileDiagnostic>;

// This is here for backwards compatibility
// with the restricted version which only uses one file
pub fn compile_file(
    context: &mut Context,
    root_file: &Path,
) -> Result<(CompiledProgram, Warnings), ErrorsAndWarnings> {
    let crate_id = prepare_crate(context, root_file);
    compile_main(context, crate_id, &CompileOptions::default())
}

/// Adds the file from the file system at `Path` to the crate graph as a root file
pub fn prepare_crate(context: &mut Context, file_name: &Path) -> CrateId {
    let path_to_std_lib_file = Path::new(STD_CRATE_NAME).join("lib.nr");
    let std_file_id = context.file_manager.add_file(&path_to_std_lib_file).unwrap();
    let std_crate_id = context.crate_graph.add_stdlib(std_file_id);

    let root_file_id = context.file_manager.add_file(file_name).unwrap();

    let root_crate_id = context.crate_graph.add_crate_root(root_file_id);

    add_dep(context, root_crate_id, std_crate_id, STD_CRATE_NAME.parse().unwrap());

    root_crate_id
}

// Adds the file from the file system at `Path` to the crate graph
pub fn prepare_dependency(context: &mut Context, file_name: &Path) -> CrateId {
    let root_file_id = context.file_manager.add_file(file_name).unwrap();

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
pub fn check_crate(
    context: &mut Context,
    crate_id: CrateId,
    deny_warnings: bool,
) -> Result<Warnings, ErrorsAndWarnings> {
    let mut errors = vec![];
    CrateDefMap::collect_defs(crate_id, context, &mut errors);

    // Check that private functions defined in another module are not called from the root crate
    if matches!(crate_id, CrateId::Root(_)) {
        let mut visibility = FunctionVisibility::default();
        if let Some(main) = context.get_main_function(&crate_id) {
            visibility.check_visibility(&context.def_interner, &main);
        } else {
            // no main function so we are in a contract
            // we check all functions
            let local_crate = context.def_map(&crate_id).unwrap();
            for (_, module) in local_crate.modules() {
                for def in module.value_definitions() {
                    if let noirc_frontend::hir::def_map::ModuleDefId::FunctionId(id) = def {
                        visibility.check_visibility(&context.def_interner, &id);
                    }
                }
            }
        }
        errors.extend(visibility.errors);
    }

    if has_errors(&errors, deny_warnings) {
        Err(errors)
    } else {
        Ok(errors)
    }
}

pub fn compute_function_abi(
    context: &Context,
    crate_id: &CrateId,
) -> Option<(Vec<AbiParameter>, Option<AbiType>)> {
    let main_function = context.get_main_function(crate_id)?;

    let func_meta = context.def_interner.function_meta(&main_function);

    let (parameters, return_type) = func_meta.into_function_signature();
    let parameters = into_abi_params(context, parameters);
    let return_type = return_type.map(|typ| AbiType::from_type(context, &typ));
    Some((parameters, return_type))
}

/// Run the frontend to check the crate for errors then compile the main function if there were none
///
/// On success this returns the compiled program alongside any warnings that were found.
/// On error this returns the non-empty list of warnings and errors.
pub fn compile_main(
    context: &mut Context,
    crate_id: CrateId,
    options: &CompileOptions,
) -> Result<(CompiledProgram, Warnings), ErrorsAndWarnings> {
    let warnings = check_crate(context, crate_id, options.deny_warnings)?;

    let main = match context.get_main_function(&crate_id) {
        Some(m) => m,
        None => {
            // TODO(#2155): This error might be a better to exist in Nargo
            let err = CustomDiagnostic::from_message(
                "cannot compile crate into a program as it does not contain a `main` function",
            )
            .in_file(FileId::default());
            return Err(vec![err]);
        }
    };

    let compiled_program = compile_no_check(context, options, main)?;

    if options.print_acir {
        println!("Compiled ACIR for main (unoptimized):");
        println!("{}", compiled_program.circuit);
    }

    Ok((compiled_program, warnings))
}

/// Run the frontend to check the crate for errors then compile all contracts if there were none
pub fn compile_contracts(
    context: &mut Context,
    crate_id: CrateId,
    options: &CompileOptions,
) -> Result<(Vec<CompiledContract>, Warnings), ErrorsAndWarnings> {
    let warnings = check_crate(context, crate_id, options.deny_warnings)?;

    // TODO: We probably want to error if contracts is empty
    let contracts = context.get_all_contracts(&crate_id);
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
                        "Compiled ACIR for {}::{} (unoptimized):",
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
    let mut errors = Vec::new();
    for function_id in &contract.functions {
        let name = context.function_name(function_id).to_owned();
        let function = match compile_no_check(context, options, *function_id) {
            Ok(function) => function,
            Err(new_error) => {
                errors.push(new_error);
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
            debug: function.debug,
        });
    }

    if errors.is_empty() {
        Ok(CompiledContract { name: contract.name, functions })
    } else {
        Err(errors)
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

    let (circuit, debug, abi) =
        create_circuit(context, program, options.show_ssa, options.show_brillig)?;

    Ok(CompiledProgram { circuit, debug, abi })
}
