use acvm::acir::circuit::OpcodeLabel;
use acvm::{acir::circuit::Circuit, Backend};
use iter_extended::try_vecmap;
use iter_extended::vecmap;
use nargo::{artifacts::contract::PreprocessedContract, NargoError};
use noirc_driver::{
    compile_contracts, compile_main, CompileOptions, CompiledProgram, ErrorsAndWarnings, Warnings,
};
use noirc_errors::reporter::ReportedErrors;
use noirc_frontend::hir::Context;
use std::path::Path;

use clap::Args;

use nargo::ops::{preprocess_contract_function, preprocess_program};

use crate::{constants::TARGET_DIR, errors::CliError, resolver::resolve_root_manifest};

use super::fs::{
    common_reference_string::{
        read_cached_common_reference_string, update_common_reference_string,
        write_cached_common_reference_string,
    },
    program::{save_contract_to_file, save_program_to_file},
};
use super::NargoConfig;

// TODO(#1388): pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";

/// Compile the program and its secret execution trace into ACIR format
#[derive(Debug, Clone, Args)]
pub(crate) struct CompileCommand {
    /// The name of the ACIR file
    circuit_name: String,

    /// Include Proving and Verification keys in the build artifacts.
    #[arg(long)]
    include_keys: bool,

    /// Compile each contract function used within the program
    #[arg(short, long)]
    contracts: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: CompileCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let circuit_dir = config.program_dir.join(TARGET_DIR);

    let mut common_reference_string = read_cached_common_reference_string();

    // If contracts is set we're compiling every function in a 'contract' rather than just 'main'.
    if args.contracts {
        let (mut context, crate_id) = resolve_root_manifest(&config.program_dir, None)?;

        let result = compile_contracts(&mut context, crate_id, &args.compile_options);
        let contracts = report_errors(result, &context, args.compile_options.deny_warnings)?;

        // TODO(#1389): I wonder if it is incorrect for nargo-core to know anything about contracts.
        // As can be seen here, It seems like a leaky abstraction where ContractFunctions (essentially CompiledPrograms)
        // are compiled via nargo-core and then the PreprocessedContract is constructed here.
        // This is due to EACH function needing it's own CRS, PKey, and VKey from the backend.
        let preprocessed_contracts: Result<Vec<PreprocessedContract>, CliError<B>> =
            try_vecmap(contracts, |contract| {
                let preprocessed_contract_functions =
                    try_vecmap(contract.functions, |mut func| {
                        func.bytecode = optimize_circuit(backend, func.bytecode)?.0;
                        common_reference_string = update_common_reference_string(
                            backend,
                            &common_reference_string,
                            &func.bytecode,
                        )
                        .map_err(CliError::CommonReferenceStringError)?;

                        preprocess_contract_function(
                            backend,
                            args.include_keys,
                            &common_reference_string,
                            func,
                        )
                        .map_err(CliError::ProofSystemCompilerError)
                    })?;

                Ok(PreprocessedContract {
                    name: contract.name,
                    backend: String::from(BACKEND_IDENTIFIER),
                    functions: preprocessed_contract_functions,
                })
            });
        for contract in preprocessed_contracts? {
            save_contract_to_file(
                &contract,
                &format!("{}-{}", &args.circuit_name, contract.name),
                &circuit_dir,
            );
        }
    } else {
        let (program, _) =
            compile_circuit(backend, None, &config.program_dir, &args.compile_options)?;
        common_reference_string =
            update_common_reference_string(backend, &common_reference_string, &program.circuit)
                .map_err(CliError::CommonReferenceStringError)?;

        let (preprocessed_program, _) =
            preprocess_program(backend, args.include_keys, &common_reference_string, program)
                .map_err(CliError::ProofSystemCompilerError)?;
        save_program_to_file(&preprocessed_program, &args.circuit_name, circuit_dir);
    }

    write_cached_common_reference_string(&common_reference_string);

    Ok(())
}

pub(crate) fn compile_circuit<B: Backend>(
    backend: &B,
    package: Option<String>,
    program_dir: &Path,
    compile_options: &CompileOptions,
) -> Result<(CompiledProgram, Context), CliError<B>> {
    let (mut context, crate_id) = resolve_root_manifest(program_dir, package)?;
    let result = compile_main(&mut context, crate_id, compile_options);
    let mut program = report_errors(result, &context, compile_options.deny_warnings)?;

    // Apply backend specific optimizations.
    let (optimized_circuit, opcode_labels) = optimize_circuit(backend, program.circuit)
        .expect("Backend does not support an opcode that is in the IR");

    program.circuit = optimized_circuit;
    let opcode_ids = vecmap(opcode_labels, |label| match label {
        OpcodeLabel::Unresolved => {
            unreachable!("Compiled circuit opcodes must resolve to some index")
        }
        OpcodeLabel::Resolved(index) => index as usize,
    });
    program.debug.update_acir(opcode_ids);

    Ok((program, context))
}

pub(super) fn optimize_circuit<B: Backend>(
    backend: &B,
    circuit: Circuit,
) -> Result<(Circuit, Vec<OpcodeLabel>), CliError<B>> {
    let result = acvm::compiler::compile(circuit, backend.np_language(), |opcode| {
        backend.supports_opcode(opcode)
    })
    .map_err(|_| NargoError::CompilationError)?;

    Ok(result)
}

/// Helper function for reporting any errors in a Result<(T, Warnings), ErrorsAndWarnings>
/// structure that is commonly used as a return result in this file.
pub(crate) fn report_errors<T>(
    result: Result<(T, Warnings), ErrorsAndWarnings>,
    context: &Context,
    deny_warnings: bool,
) -> Result<T, ReportedErrors> {
    let (t, warnings) = result.map_err(|errors| {
        noirc_errors::reporter::report_all(&context.file_manager, &errors, deny_warnings)
    })?;

    noirc_errors::reporter::report_all(&context.file_manager, &warnings, deny_warnings);
    Ok(t)
}
