use std::{collections::BTreeSet, path::PathBuf};

use acir::{AcirField, circuit::Circuit, native_types::Witness};
use acvm::compiler::find_dead_witnesses;
use clap::Args;
use noirc_artifacts::program::ProgramArtifact;

use crate::{
    Artifact,
    commands::check_dead_witnesses_cmd::dead_witnesses_in_abi::{
        AbiParameterStatuses, AbiParameterTypeStatus, AbiStatusComputer, mark_abi_parameter_as_dead,
    },
    errors::CliError,
};

use super::parse_and_normalize_path;

mod dead_witnesses_in_abi;

/// Check for dead witnesses in a compiled program or contract artifact.
#[derive(Debug, Clone, Args)]
pub struct CheckDeadWitnessesCommand {
    /// Path to the JSON build artifact (program or contract).
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub artifact_path: PathBuf,
}

pub fn run(args: CheckDeadWitnessesCommand) -> Result<(), CliError> {
    let artifact = Artifact::read_from_file(&args.artifact_path)?;

    let program = match &artifact {
        Artifact::Program(program) => program,
        Artifact::Contract(_) => {
            return Err(CliError::Generic(
                "contract artifacts are not yet supported for dead witness checking".to_string(),
            ));
        }
    };

    let circuits = &program.bytecode.functions;

    let mut has_dead = false;

    for (i, circuit) in circuits.iter().enumerate() {
        let dead_witnesses = find_dead_witnesses(circuit);
        if !dead_witnesses.is_empty() {
            show_dead_witnesses(dead_witnesses, i, circuit, program);
            has_dead = true;
        }
    }

    if has_dead {
        return Err(CliError::DeadWitnessesFound);
    }

    Ok(())
}

fn show_dead_witnesses<F: AcirField>(
    mut dead_witnesses: BTreeSet<Witness>,
    function_number: usize,
    circuit: &Circuit<F>,
    program: &ProgramArtifact,
) {
    eprintln!("Found some dead witnesses in function {function_number}:");

    if function_number == 0 {
        // Check if any of the dead witnesses are in private parameters
        let dead_witnesses_in_private_parameters = circuit
            .private_parameters
            .intersection(&dead_witnesses)
            .copied()
            .collect::<BTreeSet<_>>();

        if !dead_witnesses_in_private_parameters.is_empty() {
            // Replicate the ABI in a structure that allows us to track the dead status of each parameter and nested type.
            let (mut abi_parameter_statuses, witness_index_to_path) =
                AbiStatusComputer::default().compute(&program.abi.parameters);

            // Now mark ABI parameters as dead.
            for witness in dead_witnesses_in_private_parameters {
                let path = &witness_index_to_path[&(witness.0 as usize)];
                mark_abi_parameter_as_dead(&mut abi_parameter_statuses, path, 0);

                // Since we'll show this dead witness related to a private parameter, we can remove it
                // from the set of dead witnesses so we later only show dead witnesses related to
                // non-private parameters.
                dead_witnesses.remove(&witness);
            }

            // Bubble up the "dead" status of children up to their parents
            for (_, parameter) in &mut abi_parameter_statuses {
                parameter.recursively_compute_dead_status();
            }

            eprintln!(" - Dead witnesses in program inputs:");
            show_dead_abi_parameter_statuses(&abi_parameter_statuses, &mut Vec::new());
        }
    }

    if !dead_witnesses.is_empty() {
        let dead_witnesses =
            dead_witnesses.into_iter().map(|w| w.to_string()).collect::<Vec<_>>().join(", ");
        eprintln!(" - Other dead witnesses: {{{dead_witnesses}}}");
    }
}

fn show_dead_abi_parameter_statuses(
    abi_parameter_statuses: &AbiParameterStatuses,
    path: &mut Vec<String>,
) {
    for (name, status) in abi_parameter_statuses {
        if path.is_empty() {
            path.push(name.clone());
        } else {
            path.push(format!(".{name}"));
        }

        show_dead_abi_parameter_type_status(status, path);

        path.pop();
    }
}

fn show_dead_abi_parameter_type_status(status: &AbiParameterTypeStatus, path: &mut Vec<String>) {
    match status {
        AbiParameterTypeStatus::Single(field) => {
            if field.dead {
                eprintln!("  - {} (witness w{})", path.join(""), field.witness_index);
            }
        }
        AbiParameterTypeStatus::Array(array) => {
            if array.dead {
                eprintln!(
                    "  - {} (witnesses {{w{}, .., w{}}})",
                    path.join(""),
                    array.first_witness_index,
                    array.last_witness_index
                );
            } else {
                for (index, typ) in array.types.iter().enumerate() {
                    path.push(format!("[{index}]"));
                    show_dead_abi_parameter_type_status(typ, path);
                    path.pop();
                }
            }
        }
        AbiParameterTypeStatus::Tuple(tuple) => {
            if tuple.dead {
                eprintln!(
                    "  - {} (witnesses {{w{}, .., w{}}})",
                    path.join(""),
                    tuple.first_witness_index,
                    tuple.last_witness_index
                );
            } else {
                for (index, typ) in tuple.types.iter().enumerate() {
                    path.push(format!("[{index}]"));
                    show_dead_abi_parameter_type_status(typ, path);
                    path.pop();
                }
            }
        }
        AbiParameterTypeStatus::Struct(struct_) => {
            if struct_.dead {
                eprintln!(
                    "  - {} (witnesses {{w{}, .., w{}}})",
                    path.join(""),
                    struct_.first_witness_index,
                    struct_.last_witness_index
                );
            } else {
                for (field_name, typ) in &struct_.fields {
                    path.push(format!(".{field_name}"));
                    show_dead_abi_parameter_type_status(typ, path);
                    path.pop();
                }
            }
        }
    }
}
