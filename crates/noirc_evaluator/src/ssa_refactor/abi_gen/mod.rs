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
pub(crate) fn collate_array_lengths(_abi_params: &[AbiParameter]) -> Vec<usize> {
    Vec::new()
}

/// Computes an `noirc_abi::Abi` from a function signature.
pub(crate) fn gen_abi(func_sig: FunctionSignature) -> Abi {
    let (param_witnesses, return_witnesses) = function_signature_to_witnesses(&func_sig);

    let (parameters, return_type) = func_sig;
    Abi { parameters, return_type, param_witnesses, return_witnesses }
}

/// Takes the function signature and maps each type to a range of witness indices
fn function_signature_to_witnesses(
    func_signature: &FunctionSignature,
) -> (BTreeMap<String, Vec<Witness>>, Vec<Witness>) {
    let (parameters, return_type) = func_signature;

    let mut offset = 1;
    let param_witnesses = btree_map(parameters, |param| {
        let witnesses = abi_type_to_witness_indices(&param.typ, offset);

        let num_witnesses_needed = witnesses.len() as u32;
        offset += num_witnesses_needed;

        (param.name.clone(), witnesses)
    });

    let return_witnesses = match return_type {
        Some(return_type) => abi_type_to_witness_indices(return_type, offset),
        None => Vec::new(),
    };

    (param_witnesses, return_witnesses)
}
/// Convert an AbiType to a vector of witnesses starting from the `witness_offset`
fn abi_type_to_witness_indices(typ: &AbiType, witness_offset: u32) -> Vec<Witness> {
    let num_field_elements_needed = typ.field_count();
    let index_start = witness_offset;
    let index_end = index_start + num_field_elements_needed;
    vecmap(index_start..index_end, Witness)
}
