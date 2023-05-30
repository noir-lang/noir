use std::collections::BTreeMap;

use acvm::acir::native_types::Witness;
use iter_extended::{btree_map, vecmap};
use noirc_abi::{Abi, AbiParameter, FunctionSignature};

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
