use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use acir::{
    AcirField,
    circuit::{AcirOpcodeLocation, Circuit, Opcode, brillig::BrilligOutputs},
    native_types::{Expression, Witness},
};
use acvm::compiler::find_dead_witnesses;
use clap::Args;
use noirc_artifacts::{
    debug::{DebugArtifact, DebugInfo},
    program::ProgramArtifact,
};
use noirc_errors::{CustomDiagnostic, call_stack::CallStackId};

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

    if dead_witnesses.is_empty() {
        return;
    }

    let debug_info = &program.debug_symbols.debug_infos[function_number];
    let opcode_location_to_dead_witnesses =
        compute_call_stack_id_to_dead_witnesses(&mut dead_witnesses, circuit, debug_info);

    let debug_artifact = DebugArtifact {
        debug_symbols: program.debug_symbols.debug_infos.clone(),
        file_map: program.file_map.clone(),
    };

    for (call_stack_id, witnesses) in opcode_location_to_dead_witnesses {
        let mut call_stack = debug_info.location_tree.get_call_stack(call_stack_id);
        let location = call_stack.pop().unwrap();
        let witnesses =
            witnesses.into_iter().map(|w| w.to_string()).collect::<Vec<String>>().join(", ");
        let primary_message = format!("Dead witnesses {{{witnesses}}} ({})", call_stack_id.index());
        let secondary_message = String::new();
        let mut diagnostic =
            CustomDiagnostic::simple_warning(primary_message, secondary_message, location);
        diagnostic.call_stack = call_stack;
        diagnostic.report(&debug_artifact, false);
    }

    if dead_witnesses.is_empty() {
        return;
    }

    let dead_witnesses =
        dead_witnesses.into_iter().map(|w| w.to_string()).collect::<Vec<_>>().join(", ");
    eprintln!(" - Dead witnesses with unknown locations: {{{dead_witnesses}}}");
}

fn compute_call_stack_id_to_dead_witnesses<F: AcirField>(
    dead_witnesses: &mut BTreeSet<Witness>,
    circuit: &Circuit<F>,
    debug_info: &DebugInfo,
) -> BTreeMap<CallStackId, BTreeSet<Witness>> {
    let mut call_stack_id_to_witnesses = BTreeMap::<CallStackId, BTreeSet<Witness>>::new();

    for (index, opcode) in circuit.opcodes.iter().enumerate() {
        let opcode_witnesses = opcode_witnesses(opcode);
        let intersection =
            opcode_witnesses.intersection(dead_witnesses).copied().collect::<BTreeSet<_>>();
        if intersection.is_empty() {
            continue;
        }

        let opcode_location = AcirOpcodeLocation::new(index);
        let Some(call_stack_id) = debug_info.acir_locations.get(&opcode_location) else {
            continue;
        };
        for witness in intersection {
            call_stack_id_to_witnesses.entry(*call_stack_id).or_default().insert(witness);
            dead_witnesses.remove(&witness);
        }

        if dead_witnesses.is_empty() {
            break;
        }
    }

    call_stack_id_to_witnesses
}

/// Extract all witnesses relevant to dead witnesses from an opcode.
fn opcode_witnesses<F: AcirField>(opcode: &Opcode<F>) -> BTreeSet<Witness> {
    match opcode {
        Opcode::AssertZero(expression) => expr_witnesses(expression).collect(),
        Opcode::BlackBoxFuncCall(black_box_func_call) => black_box_func_call
            .get_input_witnesses()
            .into_iter()
            .chain(black_box_func_call.get_outputs_vec())
            .collect(),
        Opcode::MemoryOp { block_id: _, op } => expr_witnesses(&op.index)
            .chain(expr_witnesses(&op.value))
            .chain(expr_witnesses(&op.operation))
            .collect(),
        Opcode::MemoryInit { block_id: _, init, block_type: _ } => init.iter().copied().collect(),
        Opcode::BrilligCall { id: _, inputs: _, outputs, predicate: _ } => {
            let mut witnesses = BTreeSet::new();
            for output in outputs {
                match output {
                    BrilligOutputs::Simple(witness) => {
                        witnesses.insert(*witness);
                    }
                    BrilligOutputs::Array(arr) => {
                        witnesses.extend(arr.iter().copied());
                    }
                }
            }
            witnesses
        }
        Opcode::Call { id: _, inputs, outputs, predicate: _ } => {
            inputs.iter().copied().chain(outputs.iter().copied()).collect()
        }
    }
}

/// Extract all witnesses from an expression.
fn expr_witnesses<F>(expr: &Expression<F>) -> impl Iterator<Item = Witness> + '_ {
    expr.mul_terms
        .iter()
        .flat_map(|i| [i.1, i.2])
        .chain(expr.linear_combinations.iter().map(|i| i.1))
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
