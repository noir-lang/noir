use std::collections::BTreeMap;

use acvm::acir::native_types::Witness;
use iter_extended::{btree_map, vecmap};
use noirc_abi::{Abi, AbiParameter, AbiType, FunctionSignature};

/// Traverses the parameters to the program to infer the lengths of any arrays that occur.
///
/// This is needed for the acir_gen pass, because while the SSA representation of the program
/// knows the positions at which any arrays occur in the parameters to main, it does not know the
/// lengths of said arrays.
///
/// This function returns the lengths ordered such as to correspond to the ordering used by the
/// SSA representation. This allows the lengths to be consumed as array params are encountered in
/// the SSA.
pub(crate) fn collate_array_lengths(abi_params: &[AbiParameter]) -> Vec<usize> {
    let mut acc = Vec::new();
    for abi_param in abi_params {
        collate_array_lengths_recursive(&mut acc, &abi_param.typ);
    }
    acc
}

/// The underlying recursive implementation of `collate_array_lengths`
///
/// This does a depth-first traversal of the abi until an array (or string) is encountered, at
/// which point arrays are handled differently depending on the element type:
/// - arrays of fields, integers or booleans produce an array of the specified length
/// - arrays of structs produce an array of the specified length for each field of the flatten
///   struct (which reflects a simplification made during monomorphization)
fn collate_array_lengths_recursive(acc: &mut Vec<usize>, abi_type: &AbiType) {
    match abi_type {
        AbiType::Array { length, typ: elem_type } => {
            match elem_type.as_ref() {
                AbiType::Array { .. } => {
                    unreachable!("2D arrays are not supported");
                }
                AbiType::Struct { .. } => {
                    // monomorphization converts arrays of structs into an array per flattened
                    // struct field.
                    let array_count = elem_type.field_count();
                    for _ in 0..array_count {
                        acc.push(*length as usize);
                    }
                }
                AbiType::String { .. } => {
                    unreachable!("Arrays of strings are not supported");
                }
                AbiType::Boolean | AbiType::Field | AbiType::Integer { .. } => {
                    // Simple 1D array
                    acc.push(*length as usize);
                }
            }
        }
        AbiType::Struct { fields } => {
            for (_, field_type) in fields {
                collate_array_lengths_recursive(acc, field_type);
            }
        }
        AbiType::String { length } => {
            acc.push(*length as usize);
        }
        AbiType::Boolean | AbiType::Field | AbiType::Integer { .. } => {
            // Do not produce arrays
        }
    }
}

/// Arranges a function signature and a generated circuit's return witnesses into a
/// `noirc_abi::Abi`.
pub(crate) fn gen_abi(func_sig: FunctionSignature, return_witnesses: Vec<Witness>) -> Abi {
    let (parameters, return_type) = func_sig;
    let param_witnesses = param_witnesses_from_abi_param(&parameters);
    Abi { parameters, return_type, param_witnesses, return_witnesses }
}

// Takes each abi parameter and shallowly maps to the expected witness range in which the
// parameter's constituent values live.
fn param_witnesses_from_abi_param(
    abi_params: &Vec<AbiParameter>,
) -> BTreeMap<String, Vec<Witness>> {
    let mut offset = 1;
    btree_map(abi_params, |param| {
        let num_field_elements_needed = param.typ.field_count();
        let idx_start = offset;
        let idx_end = idx_start + num_field_elements_needed;
        let witnesses = vecmap(idx_start..idx_end, Witness);
        offset += num_field_elements_needed;
        (param.name.clone(), witnesses)
    })
}
