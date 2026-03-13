use std::collections::HashMap;

use iter_extended::vecmap;
use noirc_abi::{AbiParameter, AbiType};

/// Marks an ABI Parameter at the given `path[index]` as dead.
pub(super) fn mark_abi_parameter_as_dead(
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
        AbiParameterTypeStatus::Struct(struct_) => {
            mark_abi_parameter_as_dead(&mut struct_.fields, path, index + 1);
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
        AbiParameterTypeStatus::Struct(struct_) => {
            mark_abi_parameter_as_dead(&mut struct_.fields, path, index + 1);
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
pub(super) struct AbiStatusComputer {
    witness_index: usize,
    path: Vec<usize>,
    witness_index_to_path: WitnessIndexToPath,
}

impl AbiStatusComputer {
    /// Computes a replica of the ABI parameters suitable for tracking their dead status,
    /// together with a mapping of witness index to the path in the ABI parameters it corresponds to.
    pub(super) fn compute(
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

pub(super) type AbiParameterStatuses = Vec<(String, AbiParameterTypeStatus)>;

#[derive(Debug)]
pub(super) enum AbiParameterTypeStatus {
    Single(FieldAbiParameterTypeStatus),
    Array(ArrayAbiParameterTypeStatus),
    Tuple(TupleAbiParameterTypeStatus),
    Struct(StructAbiParameterTypeStatus),
}

impl AbiParameterTypeStatus {
    /// Recursively computes and sets the `dead` status of this ABI parameter type status based
    /// on its children. Returns `true` if this ABI parameter type status is dead, `false` otherwise.
    pub(super) fn recursively_compute_dead_status(&mut self) -> bool {
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
pub(super) struct FieldAbiParameterTypeStatus {
    pub(super) witness_index: usize,
    pub(super) dead: bool,
}

#[derive(Debug)]
pub(super) struct ArrayAbiParameterTypeStatus {
    pub(super) types: Vec<AbiParameterTypeStatus>,
    pub(super) first_witness_index: usize,
    pub(super) last_witness_index: usize,
    pub(super) dead: bool,
}

#[derive(Debug)]
pub(super) struct TupleAbiParameterTypeStatus {
    pub(super) types: Vec<AbiParameterTypeStatus>,
    pub(super) first_witness_index: usize,
    pub(super) last_witness_index: usize,
    pub(super) dead: bool,
}

#[derive(Debug)]
pub(super) struct StructAbiParameterTypeStatus {
    pub(super) fields: AbiParameterStatuses,
    pub(super) first_witness_index: usize,
    pub(super) last_witness_index: usize,
    pub(super) dead: bool,
}
