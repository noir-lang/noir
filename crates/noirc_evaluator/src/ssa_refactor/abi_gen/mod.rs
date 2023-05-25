use std::collections::BTreeMap;

use acvm::acir::native_types::Witness;
use noirc_abi::{Abi, AbiParameter, FunctionSignature};

pub(crate) fn collate_array_lengths(_abi_params: &[AbiParameter]) -> Vec<usize> {
    // TODO: Not needed for milestone zero, but stubbed to indicate a planned dependency
    Vec::new()
}

pub(crate) fn gen_abi(func_sig: FunctionSignature, return_witnesses: Vec<Witness>) -> Abi {
    let (parameters, return_type) = func_sig;
    let param_witnesses = param_witnesses_from_abi_param(&parameters);
    Abi { parameters, return_type, param_witnesses, return_witnesses }
}

fn param_witnesses_from_abi_param(
    abi_params: &Vec<AbiParameter>,
) -> BTreeMap<String, Vec<Witness>> {
    let mut param_witnesses = BTreeMap::new();
    let mut offset = 0;
    for param in abi_params {
        let name = param.name.clone();
        let idx_start = offset;
        let num_field_elements_needed = param.typ.field_count();
        let idx_end = idx_start + num_field_elements_needed;
        let witnesses = (idx_start..idx_end).into_iter().map(Witness).collect();
        param_witnesses.insert(name, witnesses);
        offset += num_field_elements_needed;
    }
    param_witnesses
}
