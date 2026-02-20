use std::{
    collections::{BTreeSet, HashMap},
    path::PathBuf,
};

use acir::{AcirField, circuit::Circuit, native_types::Witness};
use acvm::compiler::find_dead_witnesses;
use clap::Args;
use iter_extended::vecmap;
use noirc_abi::{AbiParameter, AbiType};
use noirc_artifacts::program::ProgramArtifact;

use crate::{Artifact, errors::CliError};

use super::parse_and_normalize_path;

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
            // Replicate the ABI in a structre that allows us to track the dead status of each parameter and nested type.
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

fn mark_abi_parameter_as_dead(
    used_abi_parameters: &mut AbiParameterStatuses,
    path: &[usize],
    index: usize,
) {
    let current_index = path[index];
    let (_name, typ) = &mut used_abi_parameters[current_index];

    match typ {
        AbiParameterTypeStatus::Single(field) => {
            field.dead = true;
        }
        AbiParameterTypeStatus::Array(array) => {
            mark_abi_parameter_type_as_dead(&mut array.types, path, index + 1);
        }
        AbiParameterTypeStatus::Tuple(tuple) => {
            mark_abi_parameter_type_as_dead(&mut tuple.types, path, index + 1);
        }
        AbiParameterTypeStatus::Struct(strukt) => {
            mark_abi_parameter_as_dead(&mut strukt.fields, path, index + 1);
        }
    }
}

fn mark_abi_parameter_type_as_dead(
    used_abi_parameter_types: &mut [AbiParameterTypeStatus],
    path: &[usize],
    index: usize,
) {
    let current_index = path[index];
    let typ = &mut used_abi_parameter_types[current_index];

    match typ {
        AbiParameterTypeStatus::Single(field) => {
            field.dead = true;
        }
        AbiParameterTypeStatus::Array(array) => {
            mark_abi_parameter_type_as_dead(&mut array.types, path, index + 1);
        }
        AbiParameterTypeStatus::Tuple(tuple) => {
            mark_abi_parameter_type_as_dead(&mut tuple.types, path, index + 1);
        }
        AbiParameterTypeStatus::Struct(strukt) => {
            mark_abi_parameter_as_dead(&mut strukt.fields, path, index + 1);
        }
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

/// Maps a witness index to the path in the ABI parameters it corresponds to.
/// For example, given this Noir program:
///
/// ```text
/// struct Foo {
///     x: [Field; 3],
///     y: Field,
/// }
///
/// fn main(foo: Foo) {}
/// ```
///
/// the private witnesses will be {w0, w1, w2, w2}, where {w0, w1, w2} correspond to `x: [Field; 3]`,
/// and `w3` corresponds to `y: Field`. So, for example, for `w1` it's path will be [0, 0, 1]:
/// - 0: for the first parameter `foo`
/// - 0: for the first field, `x`, in `Foo`
/// - 1: for the second element in the array `x`
///
/// This mapping then allows us to mark ABI parameters as used (not dead) and eventually figure
/// out which ABI parameters are dead.
type WitnessIndexToPath = HashMap<usize, Vec<usize>>;

#[derive(Default, Debug)]
struct AbiStatusComputer {
    witness_index: usize,
    path: Vec<usize>,
    witness_index_to_path: WitnessIndexToPath,
}

impl AbiStatusComputer {
    fn compute(
        mut self,
        parameters: &[AbiParameter],
    ) -> (AbiParameterStatuses, WitnessIndexToPath) {
        let params = vecmap(parameters.iter().enumerate(), |(index, parameter)| {
            self.path.push(index);
            let typ: AbiParameterTypeStatus =
                self.compute_abi_parameter_type_status(&parameter.typ);
            self.path.pop();
            (parameter.name.clone(), typ)
        });
        (params, self.witness_index_to_path)
    }

    fn compute_abi_parameter_type_status(&mut self, abi_type: &AbiType) -> AbiParameterTypeStatus {
        match abi_type {
            AbiType::Boolean | AbiType::Integer { .. } | AbiType::Field => {
                let current_index = self.witness_index;
                self.witness_index_to_path.insert(current_index, self.path.clone());
                self.witness_index += 1;

                AbiParameterTypeStatus::Single(FieldAbiParameterTypeStatus {
                    witness_index: current_index,
                    dead: false,
                })
            }
            AbiType::Array { length, typ } => {
                let first_witness_index = self.witness_index;
                let types = vecmap(0..*length, |index| {
                    self.path.push(index as usize);
                    let typ = self.compute_abi_parameter_type_status(typ);
                    self.path.pop();
                    typ
                });
                let last_witness_index = self.witness_index.saturating_sub(1);
                AbiParameterTypeStatus::Array(ArrayAbiParameterTypeStatus {
                    types,
                    first_witness_index,
                    last_witness_index,
                    dead: false,
                })
            }
            AbiType::Tuple { fields } => {
                let first_witness_index = self.witness_index;
                let types = vecmap(fields.iter().enumerate(), |(index, typ)| {
                    self.path.push(index);
                    let typ = self.compute_abi_parameter_type_status(typ);
                    self.path.pop();
                    typ
                });
                let last_witness_index = self.witness_index.saturating_sub(1);
                AbiParameterTypeStatus::Tuple(TupleAbiParameterTypeStatus {
                    types,
                    first_witness_index,
                    last_witness_index,
                    dead: false,
                })
            }
            AbiType::String { length } => {
                let first_witness_index = self.witness_index;
                let types = vecmap(0..*length, |index| {
                    self.path.push(index as usize);

                    let current_index = self.witness_index;
                    self.witness_index_to_path.insert(current_index, self.path.clone());
                    self.witness_index += 1;

                    let typ = AbiParameterTypeStatus::Single(FieldAbiParameterTypeStatus {
                        witness_index: current_index,
                        dead: false,
                    });
                    self.path.pop();
                    typ
                });
                let last_witness_index = self.witness_index.saturating_sub(1);
                AbiParameterTypeStatus::Array(ArrayAbiParameterTypeStatus {
                    types,
                    first_witness_index,
                    last_witness_index,
                    dead: false,
                })
            }
            AbiType::Struct { path: _, fields } => {
                let first_witness_index = self.witness_index;
                let fields = vecmap(fields.iter().enumerate(), |(index, (field_name, typ))| {
                    self.path.push(index);
                    let typ = self.compute_abi_parameter_type_status(typ);
                    self.path.pop();
                    (field_name.clone(), typ)
                });
                let last_witness_index = self.witness_index.saturating_sub(1);
                AbiParameterTypeStatus::Struct(StructAbiParameterTypeStatus {
                    fields,
                    first_witness_index,
                    last_witness_index,
                    dead: false,
                })
            }
        }
    }
}

type AbiParameterStatuses = Vec<(String, AbiParameterTypeStatus)>;

#[derive(Debug)]
enum AbiParameterTypeStatus {
    Single(FieldAbiParameterTypeStatus),
    Array(ArrayAbiParameterTypeStatus),
    Tuple(TupleAbiParameterTypeStatus),
    Struct(StructAbiParameterTypeStatus),
}

impl AbiParameterTypeStatus {
    /// Recursively computes and sets the `dead` status of this ABI parameter type status based
    /// on its children. Returns `true` if this ABI parameter type status is dead, `false` otherwise.
    fn recursively_compute_dead_status(&mut self) -> bool {
        match self {
            AbiParameterTypeStatus::Single(field) => field.dead,
            AbiParameterTypeStatus::Array(array) => {
                let mut all_dead = true;
                for typ in &mut array.types {
                    if !typ.recursively_compute_dead_status() {
                        all_dead = false;
                    }
                }
                array.dead = all_dead;
                all_dead
            }
            AbiParameterTypeStatus::Tuple(tuple) => {
                let mut all_dead = true;
                for typ in &mut tuple.types {
                    if !typ.recursively_compute_dead_status() {
                        all_dead = false;
                    }
                }
                tuple.dead = all_dead;
                all_dead
            }
            AbiParameterTypeStatus::Struct(struct_) => {
                let mut all_dead = true;
                for (_, typ) in &mut struct_.fields {
                    if !typ.recursively_compute_dead_status() {
                        all_dead = false;
                    }
                }
                struct_.dead = all_dead;
                all_dead
            }
        }
    }
}

#[derive(Debug)]
struct FieldAbiParameterTypeStatus {
    witness_index: usize,
    dead: bool,
}

#[derive(Debug)]
struct ArrayAbiParameterTypeStatus {
    types: Vec<AbiParameterTypeStatus>,
    first_witness_index: usize,
    last_witness_index: usize,
    dead: bool,
}

#[derive(Debug)]
struct TupleAbiParameterTypeStatus {
    types: Vec<AbiParameterTypeStatus>,
    first_witness_index: usize,
    last_witness_index: usize,
    dead: bool,
}

#[derive(Debug)]
struct StructAbiParameterTypeStatus {
    fields: AbiParameterStatuses,
    first_witness_index: usize,
    last_witness_index: usize,
    dead: bool,
}
