use std::collections::BTreeMap;

use acvm::acir::circuit::ErrorSelector;
use acvm::AcirField;
use iter_extended::vecmap;
use noirc_abi::{
    Abi, AbiErrorType, AbiParameter, AbiReturnType, AbiType, AbiValue, AbiVisibility, Sign,
};
use noirc_errors::Span;
use noirc_evaluator::ErrorType;
use noirc_frontend::ast::{Signedness, Visibility};
use noirc_frontend::TypeBinding;
use noirc_frontend::{
    hir::Context,
    hir_def::{
        expr::{HirArrayLiteral, HirExpression, HirLiteral},
        function::Param,
        stmt::HirPattern,
        types::Type,
    },
    node_interner::{FuncId, NodeInterner},
};

/// Arranges a function signature and a generated circuit's return witnesses into a
/// `noirc_abi::Abi`.
pub(super) fn gen_abi(
    context: &Context,
    func_id: &FuncId,
    return_visibility: Visibility,
    error_types: BTreeMap<ErrorSelector, ErrorType>,
) -> Abi {
    let (parameters, return_type) = compute_function_abi(context, func_id);
    let return_type = return_type.map(|typ| AbiReturnType {
        abi_type: typ,
        visibility: to_abi_visibility(return_visibility),
    });
    let error_types = error_types
        .into_iter()
        .map(|(selector, typ)| (selector, build_abi_error_type(context, typ)))
        .collect();
    Abi { parameters, return_type, error_types }
}

// Get the Span of the root crate's main function, or else a dummy span if that fails
fn get_main_function_span(context: &Context) -> Span {
    if let Some(func_id) = context.get_main_function(context.root_crate_id()) {
        context.function_meta(&func_id).location.span
    } else {
        Span::default()
    }
}

fn build_abi_error_type(context: &Context, typ: ErrorType) -> AbiErrorType {
    match typ {
        ErrorType::Dynamic(typ) => {
            if let Type::FmtString(len, item_types) = typ {
                let span = get_main_function_span(context);
                let length = len.evaluate_to_u32(span).expect("Cannot evaluate fmt length");
                let Type::Tuple(item_types) = item_types.as_ref() else {
                    unreachable!("FmtString items must be a tuple")
                };
                let item_types =
                    item_types.iter().map(|typ| abi_type_from_hir_type(context, typ)).collect();
                AbiErrorType::FmtString { length, item_types }
            } else {
                AbiErrorType::Custom(abi_type_from_hir_type(context, &typ))
            }
        }
        ErrorType::String(string) => AbiErrorType::String { string },
    }
}

pub(super) fn abi_type_from_hir_type(context: &Context, typ: &Type) -> AbiType {
    match typ {
        Type::FieldElement => AbiType::Field,
        Type::Array(size, typ) => {
            let span = get_main_function_span(context);
            let length = size
                .evaluate_to_u32(span)
                .expect("Cannot have variable sized arrays as a parameter to main");
            let typ = typ.as_ref();
            AbiType::Array { length, typ: Box::new(abi_type_from_hir_type(context, typ)) }
        }
        Type::Integer(sign, bit_width) => {
            let sign = match sign {
                Signedness::Unsigned => Sign::Unsigned,
                Signedness::Signed => Sign::Signed,
            };

            AbiType::Integer { sign, width: (*bit_width).into() }
        }
        Type::TypeVariable(binding) => {
            if binding.is_integer() || binding.is_integer_or_field() {
                match &*binding.borrow() {
                    TypeBinding::Bound(typ) => abi_type_from_hir_type(context, typ),
                    TypeBinding::Unbound(_id, _kind) => {
                        abi_type_from_hir_type(context, &Type::default_int_or_field_type())
                    }
                }
            } else {
                unreachable!("{typ} cannot be used in the abi")
            }
        }
        Type::Bool => AbiType::Boolean,
        Type::String(size) => {
            let span = get_main_function_span(context);
            let size = size
                .evaluate_to_u32(span)
                .expect("Cannot have variable sized strings as a parameter to main");
            AbiType::String { length: size }
        }

        Type::Struct(def, args) => {
            let struct_type = def.borrow();
            let fields = struct_type.get_fields(args);
            let fields =
                vecmap(fields, |(name, typ)| (name, abi_type_from_hir_type(context, &typ)));
            // For the ABI, we always want to resolve the struct paths from the root crate
            let path = context.fully_qualified_struct_path(context.root_crate_id(), struct_type.id);
            AbiType::Struct { fields, path }
        }
        Type::Alias(def, args) => abi_type_from_hir_type(context, &def.borrow().get_type(args)),
        Type::CheckedCast { to, .. } => abi_type_from_hir_type(context, to),
        Type::Tuple(fields) => {
            let fields = vecmap(fields, |typ| abi_type_from_hir_type(context, typ));
            AbiType::Tuple { fields }
        }
        Type::Error
        | Type::Unit
        | Type::Constant(..)
        | Type::InfixExpr(..)
        | Type::TraitAsType(..)
        | Type::NamedGeneric(..)
        | Type::Forall(..)
        | Type::Quoted(_)
        | Type::Slice(_)
        | Type::Function(_, _, _, _) => unreachable!("{typ} cannot be used in the abi"),
        Type::FmtString(_, _) => unreachable!("format strings cannot be used in the abi"),
        Type::MutableReference(_) => unreachable!("&mut cannot be used in the abi"),
    }
}

fn to_abi_visibility(value: Visibility) -> AbiVisibility {
    match value {
        Visibility::Public => AbiVisibility::Public,
        Visibility::Private => AbiVisibility::Private,
        Visibility::CallData(_) | Visibility::ReturnData => AbiVisibility::DataBus,
    }
}

pub(super) fn compute_function_abi(
    context: &Context,
    func_id: &FuncId,
) -> (Vec<AbiParameter>, Option<AbiType>) {
    let func_meta = context.def_interner.function_meta(func_id);

    let (parameters, return_type) = func_meta.function_signature();
    let parameters = into_abi_params(context, parameters);
    let return_type = return_type.map(|typ| abi_type_from_hir_type(context, &typ));
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
        let as_abi = abi_type_from_hir_type(context, &typ);
        AbiParameter { name: param_name, typ: as_abi, visibility: to_abi_visibility(vis) }
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
            HirLiteral::Integer(field, sign) => AbiValue::Integer { value: field.to_hex(), sign },
            _ => unreachable!("Literal cannot be used in the abi"),
        },
        _ => unreachable!("Type cannot be used in the abi {:?}", expression),
    }
}
