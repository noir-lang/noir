use std::collections::BTreeMap;

use acvm::acir::native_types::Witness;
use iter_extended::{btree_map, vecmap};
use noirc_abi::{Abi, AbiParameter, AbiType};
use noirc_frontend::{hir_def::function::Param, node_interner::NodeInterner, Type};

/// Arranges a function signature and a generated circuit's return witnesses into a
/// `noirc_abi::Abi`.
pub(crate) fn gen_abi(
    interner: &NodeInterner,
    func_sig: (Vec<Param>, Option<Type>),
    input_witnesses: &[Witness],
    return_witnesses: Vec<Witness>,
) -> Abi {
    let (parameters, return_type) = func_sig;
    let parameters = vecmap(parameters.iter(), |(pattern, typ, vis)| {
        let param_name = pattern
            .get_param_name(interner)
            .expect("Abi for tuple and struct parameters is unimplemented")
            .to_owned();
        AbiParameter { name: param_name, typ: AbiType::from(typ), visibility: vis.into() }
    });
    let return_type = return_type.map(Into::into);
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
