use std::collections::BTreeMap;

use acvm::acir::native_types::Witness;
use iter_extended::{btree_map, vecmap};
use noirc_abi::{Abi, AbiParameter};
use noirc_frontend::{
    hir_def::{
        function::{FunctionSignature, Param},
        stmt::HirPattern,
    },
    node_interner::NodeInterner,
};

/// Attempts to retrieve the name of this parameter. Returns None
/// if this parameter is a tuple or struct pattern.
fn get_param_name<'a>(pattern: &HirPattern, interner: &'a NodeInterner) -> Option<&'a str> {
    match pattern {
        HirPattern::Identifier(ident) => Some(interner.definition_name(ident.id)),
        HirPattern::Mutable(pattern, _) => get_param_name(pattern, interner),
        HirPattern::Tuple(_, _) => None,
        HirPattern::Struct(_, _, _) => None,
    }
}

pub fn into_abi_params(params: Vec<Param>, interner: &NodeInterner) -> Vec<AbiParameter> {
    vecmap(params, |param| {
        let param_name = get_param_name(&param.0, interner)
            .expect("Abi for tuple and struct parameters is unimplemented")
            .to_owned();
        let as_abi = param.1.as_abi_type();
        AbiParameter { name: param_name, typ: as_abi, visibility: param.2.into() }
    })
}

/// Arranges a function signature and a generated circuit's return witnesses into a
/// `noirc_abi::Abi`.
pub(crate) fn gen_abi(
    interner: &NodeInterner,
    func_sig: FunctionSignature,
    input_witnesses: &[Witness],
    return_witnesses: Vec<Witness>,
) -> Abi {
    let (parameters, return_type) = func_sig;
    let parameters = into_abi_params(parameters, interner);
    let return_type = return_type.map(|typ| typ.as_abi_type());
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
