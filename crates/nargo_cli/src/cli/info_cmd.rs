use acvm::Backend;
use clap::Args;

use noirc_driver::{compile_contracts, CompileOptions};
use std::path::Path;

use crate::cli::compile_cmd::compile_circuit;
use crate::errors::CliError;
use crate::resolver::resolve_root_manifest;

use super::compile_cmd::report_errors;

use super::NargoConfig;

/// Provides detailed informaton on a circuit
/// Current information provided:
/// 1. The number of ACIR opcodes
/// 2. Counts the final number gates in the circuit used by a backend
#[derive(Debug, Clone, Args)]
pub(crate) struct InfoCommand {
    /// Get information of all contracts used within the program
    #[arg(short, long)]
    contracts: bool,

    /// Get information of a contract used within the program
    #[arg(short, long)]
    contract: Option<String>,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: InfoCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    count_opcodes_and_gates_with_path(backend, config.program_dir, args)
}

fn count_opcodes_and_gates_with_path<B: Backend, P: AsRef<Path>>(
    backend: &B,
    program_dir: P,
    args: InfoCommand,
) -> Result<(), CliError<B>> {
    if args.contracts {
        let (mut context, crate_id) = resolve_root_manifest(program_dir.as_ref(), None)?;

        let result = compile_contracts(&mut context, crate_id, &args.compile_options);
        let contracts = report_errors(result, &context, args.compile_options.deny_warnings)?;

        let mut total_num_opcodes_in_all_contracts = 0;
        let mut total_num_circuit_size_in_all_contracts = 0;

        for contract in contracts {
            let mut total_num_opcodes = 0;
            let mut total_circuit_size = 0;
            let mut function_info = Vec::new();
            for function in contract.functions {
                let num_opcodes = function.bytecode.opcodes.len();
                let exact_circuit_size = backend
                    .get_exact_circuit_size(&function.bytecode)
                    .map_err(CliError::ProofSystemCompilerError)?;
                total_num_opcodes += num_opcodes;
                total_circuit_size += exact_circuit_size;
                function_info.push((function.name, num_opcodes, exact_circuit_size));
            }
            total_num_opcodes_in_all_contracts += total_num_opcodes;
            total_num_circuit_size_in_all_contracts += total_circuit_size;

            println!(
                "Total ACIR opcodes generated for language {:?} in contract {}: {}",
                backend.np_language(),
                contract.name,
                total_num_opcodes
            );
            println!("Backend circuit size for contract {}: {total_circuit_size}", contract.name);
            println!();

            for info in function_info {
                println!(
                    "Total ACIR opcodes generated of function {} for language {:?} in contract {}: {}",
                    info.0,
                    backend.np_language(),
                    contract.name,
                    info.1,
                );
                println!(
                    "Backend circuit size for function {} in contract {}: {}",
                    info.0, contract.name, info.2
                );
            }
        }

        println!();
        println!(
            "Total ACIR opcodes generated for language {:?} in all contracts: {}",
            backend.np_language(),
            total_num_opcodes_in_all_contracts
        );
        println!(
            "Backend circuit size for all contracts: {total_num_circuit_size_in_all_contracts}"
        );
    } else if args.contract.is_some() {
        let contract_name = args.contract.unwrap();
        let (mut context, crate_id) = resolve_root_manifest(program_dir.as_ref(), None)?;
        let result = compile_contracts(&mut context, crate_id, &args.compile_options);
        let contracts = report_errors(result, &context, args.compile_options.deny_warnings)?;

        for contract in contracts {
            if contract.name == contract_name {
                let mut total_num_opcodes = 0;
                let mut total_circuit_size = 0;
                let mut function_info = Vec::new();
                for function in contract.functions {
                    let num_opcodes = function.bytecode.opcodes.len();
                    let exact_circuit_size = backend
                        .get_exact_circuit_size(&function.bytecode)
                        .map_err(CliError::ProofSystemCompilerError)?;
                    total_num_opcodes += num_opcodes;
                    total_circuit_size += exact_circuit_size;
                    function_info.push((function.name, num_opcodes, exact_circuit_size));
                }

                println!(
                    "Total ACIR opcodes generated for language {:?} in contract {}: {}",
                    backend.np_language(),
                    contract.name,
                    total_num_opcodes
                );
                println!(
                    "Backend circuit size for contract {}: {total_circuit_size}",
                    contract.name
                );
                println!();

                for info in function_info {
                    println!(
                        "Total ACIR opcodes generated of function {} for language {:?} in contract {}: {}",
                        info.0,
                        backend.np_language(),
                        contract.name,
                        info.1,
                    );
                    println!(
                        "Backend circuit size for function {} in contract {}: {}",
                        info.0, contract.name, info.2
                    );
                }

                break;
            }

            println!("Cannot find contract.")
        }
    } else {
        let (compiled_program, _) =
            compile_circuit(backend, None, program_dir.as_ref(), &args.compile_options)?;
        let num_opcodes = compiled_program.circuit.opcodes.len();

        println!(
            "Total ACIR opcodes generated for language {:?}: {}",
            backend.np_language(),
            num_opcodes
        );

        let exact_circuit_size = backend
            .get_exact_circuit_size(&compiled_program.circuit)
            .map_err(CliError::ProofSystemCompilerError)?;
        println!("Backend circuit size: {exact_circuit_size}");
    }

    Ok(())
}
