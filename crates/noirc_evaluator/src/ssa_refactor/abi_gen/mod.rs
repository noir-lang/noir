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
pub(crate) fn collate_array_info(abi_params: &[AbiParameter]) -> Vec<(usize, AbiType)> {
    let mut acc = Vec::new();
    for abi_param in abi_params {
        collate_array_info_recursive(&mut acc, &abi_param.typ);
    }
    acc
}

/// The underlying recursive implementation of `collate_array_info`
///
/// This does a depth-first traversal of the abi until an array (or string) is encountered, at
/// which point arrays are handled differently depending on the element type:
/// - arrays of fields, integers or booleans produce an array of the specified length
/// - arrays of structs produce an array of the specified length for each field of the flatten
///   struct (which reflects a simplification made during monomorphization)
fn collate_array_info_recursive(acc: &mut Vec<(usize, AbiType)>, abi_type: &AbiType) {
    match abi_type {
        AbiType::Array { length, typ: elem_type } => {
            let elem_type = elem_type.as_ref();
            match elem_type {
                AbiType::Array { .. } => {
                    unreachable!("2D arrays are not supported");
                }
                AbiType::Struct { .. } => {
                    // monomorphization converts arrays of structs into an array per flattened
                    // struct field.
                    let mut destructured_array_types = Vec::new();
                    flatten_abi_type_recursive(&mut destructured_array_types, elem_type);
                    for abi_type in destructured_array_types {
                        acc.push((*length as usize, abi_type));
                    }
                }
                AbiType::String { .. } => {
                    unreachable!("Arrays of strings are not supported");
                }
                AbiType::Boolean | AbiType::Field | AbiType::Integer { .. } => {
                    // Simple 1D array
                    acc.push((*length as usize, elem_type.clone()));
                }
            }
        }
        AbiType::Struct { fields } => {
            for (_, field_type) in fields {
                collate_array_info_recursive(acc, field_type);
            }
        }
        AbiType::String { length } => {
            // Strings are implemented as u8 arrays
            let element_type = AbiType::Integer { sign: noirc_abi::Sign::Unsigned, width: 8 };
            acc.push((*length as usize, element_type));
        }
        AbiType::Boolean | AbiType::Field | AbiType::Integer { .. } => {
            // Do not produce arrays
        }
    }
}

/// Used for flattening a struct into its ordered constituent field types. This is needed for
/// informing knowing the bit widths of any array sets that were destructured from an array of
/// structs. For this reason, any array encountered within this function are considered to be
/// nested within a struct and are therefore disallowed. This is acceptable because this function
/// will only be applied to structs which have been found in an array.
fn flatten_abi_type_recursive(acc: &mut Vec<AbiType>, abi_type: &AbiType) {
    match abi_type {
        AbiType::Array { .. } | AbiType::String { .. } => {
            unreachable!("2D arrays are unsupported")
        }
        AbiType::Boolean | AbiType::Integer { .. } | AbiType::Field => acc.push(abi_type.clone()),
        AbiType::Struct { fields } => {
            for (_, field_type) in fields {
                flatten_abi_type_recursive(acc, field_type);
            }
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
