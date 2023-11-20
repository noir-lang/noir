use std::collections::BTreeMap;

use acvm::acir::native_types::Witness;
use iter_extended::{btree_map, vecmap};
use noirc_abi::{Abi, AbiParameter, AbiType};
use noirc_frontend::{
    hir::Context,
    hir_def::{
        function::{FunctionSignature, Param},
        stmt::HirPattern,
    },
    node_interner::NodeInterner,
    Visibility,
};
use std::ops::Range;

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

pub fn into_abi_params(context: &Context, params: Vec<Param>) -> Vec<AbiParameter> {
    vecmap(params, |(pattern, typ, vis)| {
        let param_name = get_param_name(&pattern, &context.def_interner)
            .expect("Abi for tuple and struct parameters is unimplemented")
            .to_owned();
        let as_abi = AbiType::from_type(context, &typ);
        AbiParameter { name: param_name, typ: as_abi, visibility: vis.into() }
    })
}

/// Arranges a function signature and a generated circuit's return witnesses into a
/// `noirc_abi::Abi`.
pub(crate) fn gen_abi(
    context: &Context,
    func_sig: FunctionSignature,
    input_witnesses: Vec<Range<Witness>>,
    return_witnesses: Vec<Witness>,
    return_visibility: Visibility,
) -> Abi {
    let (parameters, return_type) = func_sig;
    let parameters = into_abi_params(context, parameters);
    let return_type =
        return_type.map(|typ| (AbiType::from_type(context, &typ), return_visibility.into()));
    let param_witnesses = param_witnesses_from_abi_param(&parameters, input_witnesses);
    Abi { parameters, return_type, param_witnesses, return_witnesses }
}

// Takes each abi parameter and shallowly maps to the expected witness range in which the
// parameter's constituent values live.
fn param_witnesses_from_abi_param(
    abi_params: &Vec<AbiParameter>,
    input_witnesses: Vec<Range<Witness>>,
) -> BTreeMap<String, Vec<Range<Witness>>> {
    let mut idx = 0_usize;
    if input_witnesses.is_empty() {
        return BTreeMap::new();
    }
    let mut processed_range = input_witnesses[idx].start.witness_index();

    btree_map(abi_params, |param| {
        let num_field_elements_needed = param.typ.field_count();
        let mut wit = Vec::new();
        let mut processed_fields = 0;
        while processed_fields < num_field_elements_needed {
            let end = input_witnesses[idx].end.witness_index();
            if num_field_elements_needed <= end - processed_range {
                wit.push(
                    Witness(processed_range)..Witness(processed_range + num_field_elements_needed),
                );
                processed_range += num_field_elements_needed;
                processed_fields += num_field_elements_needed;
            } else {
                // consume the current range
                wit.push(Witness(processed_range)..input_witnesses[idx].end);
                processed_fields += end - processed_range;
                idx += 1;
                processed_range = input_witnesses[idx].start.witness_index();
            }
        }
        (param.name.clone(), wit)
    })
}
