use acvm::Backend;
use nargo::ops::PreprocessedProgram;
use nargo::ops::{optimize_program, preprocess_function, OptimizedProgram};
use nargo::package::Package;
use noirc_driver::{
    compile_contracts, compile_main, CompileOptions, CompiledProgram, ErrorsAndWarnings, Warnings,
};
use noirc_frontend::graph::CrateName;
use noirc_frontend::hir::Context;

use clap::Args;

use crate::errors::{CliError, CompileError};
use crate::manifest::resolve_workspace_from_toml;
use crate::{find_package_manifest, prepare_package};

use super::fs::{
    common_reference_string::{
        read_cached_common_reference_string, update_common_reference_string,
        write_cached_common_reference_string,
    },
    program::save_program_to_file,
};
use super::NargoConfig;

// TODO(#1388): pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";

/// Compile the program and its secret execution trace into ACIR format
#[derive(Debug, Clone, Args)]
pub(crate) struct CompileCommand {
    /// Include Proving and Verification keys in the build artifacts.
    #[arg(long)]
    include_keys: bool,

    /// Compile each contract function used within the program
    #[arg(short, long)]
    contracts: bool,

    /// The name of the package to compile
    #[clap(long)]
    package: Option<CrateName>,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: CompileCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let toml_path = find_package_manifest(&config.program_dir)?;
    let workspace = resolve_workspace_from_toml(&toml_path, args.package)?;
    let circuit_dir = workspace.target_directory_path();

    let mut common_reference_string = read_cached_common_reference_string();

    // If contracts is set we're compiling every function in a 'contract' rather than just 'main'.
    if args.contracts {
        for package in &workspace {
            let (mut context, crate_id) = prepare_package(package);
            let result = compile_contracts(&mut context, crate_id, &args.compile_options);
            let compiled_program =
                report_errors(result, &context, args.compile_options.deny_warnings)?;

            let optimized_program = optimize_program(backend, compiled_program)?;

            let preprocessed_program = preprocess_program(
                backend,
                common_reference_string,
                optimized_program,
                args.include_keys,
            )?;

            save_program_to_file(
                &preprocessed_program,
                &format!(
                    "{}_{}",
                    package.name,
                    // TODO: Turn into proper error
                    compiled_program.name.expect("Contract needs a name")
                )
                .parse()
                .expect("Valid crate name"),
                &circuit_dir,
            );
        }
    } else {
        for package in &workspace {
            let (_, compiled_program) = compile_package(backend, package, &args.compile_options)?;
            let optimized_program = optimize_program(backend, compiled_program)?;

            let preprocessed_program = preprocess_program(
                backend,
                common_reference_string,
                optimized_program,
                args.include_keys,
            )?;

            save_program_to_file(&preprocessed_program, &package.name, &circuit_dir);
        }
    }

    write_cached_common_reference_string(&common_reference_string);

    Ok(())
}

pub(crate) fn compile_package<B: Backend>(
    backend: &B,
    package: &Package,
    compile_options: &CompileOptions,
) -> Result<(Context, CompiledProgram), CompileError> {
    if package.is_library() {
        return Err(CompileError::LibraryCrate(package.name.clone()));
    }

    let (mut context, crate_id) = prepare_package(package);
    let result = compile_main(&mut context, crate_id, compile_options);
    let mut program = report_errors(result, &context, compile_options.deny_warnings)?;

    Ok((context, program))
}

/// Helper function for reporting any errors in a Result<(T, Warnings), ErrorsAndWarnings>
/// structure that is commonly used as a return result in this file.
pub(crate) fn report_errors<T>(
    result: Result<(T, Warnings), ErrorsAndWarnings>,
    context: &Context,
    deny_warnings: bool,
) -> Result<T, CompileError> {
    let (t, warnings) = result.map_err(|errors| {
        noirc_errors::reporter::report_all(&context.file_manager, &errors, deny_warnings)
    })?;

    noirc_errors::reporter::report_all(&context.file_manager, &warnings, deny_warnings);
    Ok(t)
}

pub(crate) fn preprocess_program<B: Backend>(
    backend: &B,
    // TODO: This will become more streamlined when backends control their CRS. Then, we can move it into nargo-core
    mut common_reference_string: Vec<u8>,
    // Takes ownership so you don't use an optimized program after it has been preprocessed
    program: OptimizedProgram,
    // TODO: This will be removed when we always generate pkey and vkey everytime we preprocess
    include_keys: bool,
) -> Result<PreprocessedProgram, CliError<B>> {
    let mut preprocessed_functions = Vec::new();
    for func in program.functions {
        update_common_reference_string(backend, common_reference_string, &func.bytecode)
            .map_err(CliError::CommonReferenceStringError)?;

        let preprocessed_func =
            preprocess_function(backend, include_keys, &common_reference_string, func)
                .map_err(CliError::ProofSystemCompilerError)?;

        preprocessed_functions.push(preprocessed_func);
    }

    let preprocessed_program = PreprocessedProgram {
        name: program.name,
        backend: String::from(BACKEND_IDENTIFIER),
        functions: preprocessed_functions,
    };

    Ok(preprocessed_program)
}
