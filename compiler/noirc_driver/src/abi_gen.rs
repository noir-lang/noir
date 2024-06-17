use std::collections::BTreeMap;

use acvm::acir::circuit::ErrorSelector;
use iter_extended::vecmap;
use noirc_abi::{Abi, AbiErrorType, AbiParameter, AbiReturnType, AbiType, AbiValue};
use noirc_frontend::ast::Visibility;
use noirc_frontend::{
    hir::Context,
    hir_def::{expr::HirArrayLiteral, function::Param, stmt::HirPattern, types::Type},
    macros_api::{HirExpression, HirLiteral},
    node_interner::{FuncId, NodeInterner},
};

/// Arranges a function signature and a generated circuit's return witnesses into a
/// `noirc_abi::Abi`.
pub(super) fn gen_abi(
    context: &Context,
    func_id: &FuncId,
    return_visibility: Visibility,
    error_types: BTreeMap<ErrorSelector, Type>,
) -> Abi {
    let (parameters, return_type) = compute_function_abi(context, func_id);
    let return_type = return_type
        .map(|typ| AbiReturnType { abi_type: typ, visibility: return_visibility.into() });
    let error_types = error_types
        .into_iter()
        .map(|(selector, typ)| (selector, AbiErrorType::from_type(context, &typ)))
        .collect();
    Abi { parameters, return_type, error_types }
}

pub(super) fn compute_function_abi(
    context: &Context,
    func_id: &FuncId,
) -> (Vec<AbiParameter>, Option<AbiType>) {
    let func_meta = context.def_interner.function_meta(func_id);

    let (parameters, return_type) = func_meta.function_signature();
    let parameters = into_abi_params(context, parameters);
    let return_type = return_type.map(|typ| AbiType::from_type(context, &typ));
    (parameters, return_type)
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

pub(super) fn value_from_hir_expression(context: &Context, expression: HirExpression) -> AbiValue {
    match expression {
        HirExpression::Tuple(expr_ids) => {
            let fields = expr_ids
                .iter()
                .map(|expr_id| {
                    value_from_hir_expression(context, context.def_interner.expression(expr_id))
                })
                .collect();
            AbiValue::Tuple { fields }
        }
        HirExpression::Constructor(constructor) => {
            let fields = constructor
                .fields
                .iter()
                .map(|(ident, expr_id)| {
                    (
                        ident.0.contents.to_string(),
                        value_from_hir_expression(
                            context,
                            context.def_interner.expression(expr_id),
                        ),
                    )
                })
                .collect();
            AbiValue::Struct { fields }
        }
        HirExpression::Literal(literal) => match literal {
            HirLiteral::Array(hir_array) => match hir_array {
                HirArrayLiteral::Standard(expr_ids) => {
                    let value = expr_ids
                        .iter()
                        .map(|expr_id| {
                            value_from_hir_expression(
                                context,
                                context.def_interner.expression(expr_id),
                            )
                        })
                        .collect();
                    AbiValue::Array { value }
                }
                _ => unreachable!("Repeated arrays cannot be used in the abi"),
            },
            HirLiteral::Bool(value) => AbiValue::Boolean { value },
            HirLiteral::Str(value) => AbiValue::String { value },
            HirLiteral::Integer(field, sign) => {
                AbiValue::Integer { value: field.to_string(), sign }
            }
            _ => unreachable!("Literal cannot be used in the abi"),
        },
        _ => unreachable!("Type cannot be used in the abi {:?}", expression),
    }
}
