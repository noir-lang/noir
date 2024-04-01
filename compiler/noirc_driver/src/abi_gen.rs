use std::collections::BTreeMap;

use acvm::acir::native_types::Witness;
use iter_extended::{btree_map, vecmap};
use noirc_abi::{Abi, AbiParameter, AbiReturnType, AbiType};
use noirc_frontend::{
    hir::Context,
    hir_def::{function::Param, stmt::HirPattern},
    node_interner::{FuncId, NodeInterner},
    Type,
};
use std::ops::Range;

/// Arranges a function signature and a generated circuit's return witnesses into a
/// `noirc_abi::Abi`.
pub(super) fn gen_abi(
    context: &Context,
    func_id: &FuncId,
    input_witnesses: Vec<Witness>,
    return_witnesses: Vec<Witness>,
) -> Abi {
    let (parameters, return_types) = compute_function_abi(context, func_id);
    let param_witnesses = param_witnesses_from_abi_param(&parameters, input_witnesses);
    let return_witnesses = return_witnesses_from_abi_returns(&return_types, return_witnesses);
    Abi { parameters, return_types, param_witnesses, return_witnesses }
}

pub(super) fn compute_function_abi(
    context: &Context,
    func_id: &FuncId,
) -> (Vec<AbiParameter>, Vec<AbiReturnType>) {
    let func_meta = context.def_interner.function_meta(func_id);

    let (parameters, return_type) = func_meta.function_signature();
    let parameters = into_abi_params(context, parameters);

    let (visibilities, return_type) = return_type;

    let return_types = match return_type {
        None => vec![],
        Some(Type::Tuple(types)) if visibilities.len() == types.len() => visibilities
            .iter()
            .copied()
            .zip(types.iter().cloned())
            .map(|(visibility, typ)| AbiReturnType {
                abi_type: AbiType::from_type(context, &typ),
                visibility: visibility.into(),
            })
            .collect(),
        Some(return_type) => vec![AbiReturnType {
            abi_type: AbiType::from_type(context, &return_type),
            visibility: visibilities[0].into(),
        }],
    };
    (parameters, return_types)
}

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

fn into_abi_params(context: &Context, params: Vec<Param>) -> Vec<AbiParameter> {
    vecmap(params, |(pattern, typ, vis)| {
        let param_name = get_param_name(&pattern, &context.def_interner)
            .expect("Abi for tuple and struct parameters is unimplemented")
            .to_owned();
        let as_abi = AbiType::from_type(context, &typ);
        AbiParameter { name: param_name, typ: as_abi, visibility: vis.into() }
    })
}

// Takes each abi parameter and shallowly maps to the expected witness range in which the
// parameter's constituent values live.
fn param_witnesses_from_abi_param(
    abi_params: &[AbiParameter],
    input_witnesses: Vec<Witness>,
) -> BTreeMap<String, Vec<Range<Witness>>> {
    let mut idx = 0_usize;
    if input_witnesses.is_empty() {
        return BTreeMap::new();
    }

    btree_map(abi_params, |param| {
        let num_field_elements_needed = param.typ.field_count() as usize;
        let param_witnesses = &input_witnesses[idx..idx + num_field_elements_needed];

        // It's likely that `param_witnesses` will consist of mostly incrementing witness indices.
        // We then want to collapse these into `Range`s to save space.
        let param_witnesses = collapse_ranges(param_witnesses);
        idx += num_field_elements_needed;
        (param.name.clone(), param_witnesses)
    })
}

// Takes each abi parameter and shallowly maps to the expected witness range in which the
// parameter's constituent values live.
fn return_witnesses_from_abi_returns(
    abi_returns: &[AbiReturnType],
    return_witnesses: Vec<Witness>,
) -> Vec<Vec<Witness>> {
    let mut idx = 0_usize;
    if return_witnesses.is_empty() {
        return vec![];
    }

    vecmap(abi_returns, |ret| {
        let num_field_elements_needed = ret.abi_type.field_count() as usize;
        let return_witnesses = &return_witnesses[idx..idx + num_field_elements_needed];

        idx += num_field_elements_needed;
        return_witnesses.to_vec()
    })
}

/// Takes a vector of [`Witnesses`][`Witness`] and collapses it into a vector of [`Range`]s of [`Witnesses`][`Witness`].
fn collapse_ranges(witnesses: &[Witness]) -> Vec<Range<Witness>> {
    if witnesses.is_empty() {
        return Vec::new();
    }
    let mut wit = Vec::new();
    let mut last_wit: Witness = witnesses[0];

    for (i, witness) in witnesses.iter().enumerate() {
        if i == 0 {
            continue;
        };
        let witness_index = witness.witness_index();
        let prev_witness_index = witnesses[i - 1].witness_index();
        if witness_index != prev_witness_index + 1 {
            wit.push(last_wit..Witness(prev_witness_index + 1));
            last_wit = *witness;
        };
    }

    let last_witness = witnesses.last().unwrap().witness_index();
    wit.push(last_wit..Witness(last_witness + 1));

    wit
}

#[cfg(test)]
mod test {
    use std::ops::Range;

    use acvm::acir::native_types::Witness;

    use super::collapse_ranges;

    #[test]
    fn collapses_single_range() {
        let witnesses: Vec<_> = vec![1, 2, 3].into_iter().map(Witness::from).collect();

        let collapsed_witnesses = collapse_ranges(&witnesses);

        assert_eq!(collapsed_witnesses, vec![Range { start: Witness(1), end: Witness(4) },]);
    }

    #[test]
    fn collapse_ranges_correctly() {
        let witnesses: Vec<_> =
            vec![1, 2, 3, 5, 6, 2, 3, 4].into_iter().map(Witness::from).collect();

        let collapsed_witnesses = collapse_ranges(&witnesses);

        assert_eq!(
            collapsed_witnesses,
            vec![
                Range { start: Witness(1), end: Witness(4) },
                Range { start: Witness(5), end: Witness(7) },
                Range { start: Witness(2), end: Witness(5) }
            ]
        );
    }
}
