use std::collections::BTreeMap;

use acvm::acir::native_types::Witness;
use iter_extended::btree_map;
use noirc_abi::{Abi, AbiParameter, FunctionSignature};

/// Arranges a function signature and a generated circuit's return witnesses into a
/// `noirc_abi::Abi`.
pub(crate) fn gen_abi(
    func_sig: FunctionSignature,
    return_witnesses: Vec<Witness>,
    input_witnesses: &[Witness],
) -> Abi {
    let (parameters, return_type) = func_sig;
    let param_witnesses = param_witnesses_from_abi_param(&parameters, input_witnesses);
    Abi { parameters, return_type, param_witnesses, return_witnesses }
}

// Takes each abi parameter and shallowly maps to the expected witness range in which the
// parameter's constituent values live.
fn param_witnesses_from_abi_param(
    abi_params: &Vec<AbiParameter>,
    input_witnesses: &[Witness],
) -> BTreeMap<String, Vec<Witness>> {
    let mut idx = 0_usize;

    btree_map(abi_params, |param| {
        let num_field_elements_needed = param.typ.field_count();
        let mut wit = Vec::new();
        for _ in 0..num_field_elements_needed {
            wit.push(input_witnesses[idx]);
            idx += 1;
        }
        (param.name.clone(), wit)
    })
}
