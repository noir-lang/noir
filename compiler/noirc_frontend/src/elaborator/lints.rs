use crate::{
    ast::FunctionKind,
    graph::CrateId,
    hir::{
        resolution::errors::{PubPosition, ResolverError},
        type_check::TypeCheckError,
    },
    hir_def::expr::HirIdent,
    macros_api::{
        HirExpression, HirLiteral, NodeInterner, NoirFunction, Signedness, UnaryOp,
        UnresolvedTypeData, Visibility,
    },
    node_interner::{DefinitionKind, ExprId, FuncId},
    Type,
};

use noirc_errors::Span;

pub(super) fn deprecated_function(interner: &NodeInterner, expr: ExprId) -> Option<TypeCheckError> {
    let HirExpression::Ident(HirIdent { location, id, impl_kind: _ }, _) =
        interner.expression(&expr)
    else {
        return None;
    };

    let Some(DefinitionKind::Function(func_id)) = interner.try_definition(id).map(|def| &def.kind)
    else {
        return None;
    };

    let attributes = interner.function_attributes(func_id);
    attributes.get_deprecated_note().map(|note| TypeCheckError::CallDeprecated {
        name: interner.definition_name(id).to_string(),
        note,
        span: location.span,
    })
}

/// Inline attributes are only relevant for constrained functions
/// as all unconstrained functions are not inlined and so
/// associated attributes are disallowed.
pub(super) fn inlining_attributes(func: &NoirFunction) -> Option<ResolverError> {
    if func.def.is_unconstrained {
        let attributes = func.attributes().clone();

        if attributes.is_no_predicates() {
            Some(ResolverError::NoPredicatesAttributeOnUnconstrained {
                ident: func.name_ident().clone(),
            })
        } else if attributes.is_foldable() {
            Some(ResolverError::FoldAttributeOnUnconstrained { ident: func.name_ident().clone() })
        } else {
            None
        }
    } else {
        None
    }
}

/// Attempting to define new low level (`#[builtin]` or `#[foreign]`) functions outside of the stdlib is disallowed.
pub(super) fn low_level_function_outside_stdlib(
    func: &NoirFunction,
    crate_id: CrateId,
) -> Option<ResolverError> {
    let is_low_level_function =
        func.attributes().function.as_ref().map_or(false, |func| func.is_low_level());
    if !crate_id.is_stdlib() && is_low_level_function {
        Some(ResolverError::LowLevelFunctionOutsideOfStdlib { ident: func.name_ident().clone() })
    } else {
        None
    }
}

/// Oracle definitions (functions with the `#[oracle]` attribute) must be marked as unconstrained.
pub(super) fn oracle_not_marked_unconstrained(func: &NoirFunction) -> Option<ResolverError> {
    let is_oracle_function =
        func.attributes().function.as_ref().map_or(false, |func| func.is_oracle());
    if is_oracle_function && !func.def.is_unconstrained {
        Some(ResolverError::OracleMarkedAsConstrained { ident: func.name_ident().clone() })
    } else {
        None
    }
}

/// Oracle functions may not be called by constrained functions directly.
///
/// In order for a constrained function to call an oracle it must first call through an unconstrained function.
pub(super) fn oracle_called_from_constrained_function(
    interner: &NodeInterner,
    called_func: &FuncId,
    calling_from_constrained_runtime: bool,
    span: Span,
) -> Option<ResolverError> {
    if !calling_from_constrained_runtime {
        return None;
    }

    let function_attributes = interner.function_attributes(called_func);
    let is_oracle_call =
        function_attributes.function.as_ref().map_or(false, |func| func.is_oracle());
    if is_oracle_call {
        Some(ResolverError::UnconstrainedOracleReturnToConstrained { span })
    } else {
        None
    }
}

/// `pub` is required on return types for entry point functions
pub(super) fn missing_pub(func: &NoirFunction, is_entry_point: bool) -> Option<ResolverError> {
    if is_entry_point
        && func.return_type().typ != UnresolvedTypeData::Unit
        && func.def.return_visibility == Visibility::Private
    {
        Some(ResolverError::NecessaryPub { ident: func.name_ident().clone() })
    } else {
        None
    }
}

/// `#[recursive]` attribute is only allowed for entry point functions
pub(super) fn recursive_non_entrypoint_function(
    func: &NoirFunction,
    is_entry_point: bool,
) -> Option<ResolverError> {
    if !is_entry_point && func.kind == FunctionKind::Recursive {
        Some(ResolverError::MisplacedRecursiveAttribute { ident: func.name_ident().clone() })
    } else {
        None
    }
}

/// Check that we are not passing a mutable reference from a constrained runtime to an unconstrained runtime.
pub(super) fn unconstrained_function_args(
    function_args: &[(Type, ExprId, Span)],
) -> Vec<TypeCheckError> {
    function_args
        .iter()
        .filter_map(|(typ, _, span)| {
            if !typ.is_valid_for_unconstrained_boundary() {
                Some(TypeCheckError::ConstrainedReferenceToUnconstrained { span: *span })
            } else {
                None
            }
        })
        .collect()
}

/// Check that we are not passing a slice from an unconstrained runtime to a constrained runtime.
pub(super) fn unconstrained_function_return(
    return_type: &Type,
    span: Span,
) -> Option<TypeCheckError> {
    if return_type.contains_slice() {
        Some(TypeCheckError::UnconstrainedSliceReturnToConstrained { span })
    } else if !return_type.is_valid_for_unconstrained_boundary() {
        Some(TypeCheckError::UnconstrainedReferenceToConstrained { span })
    } else {
        None
    }
}

/// Only entrypoint functions require a `pub` visibility modifier applied to their return types.
///
/// Application of `pub` to other functions is not meaningful and is a mistake.
pub(super) fn unnecessary_pub_return(
    func: &NoirFunction,
    is_entry_point: bool,
) -> Option<ResolverError> {
    if !is_entry_point && func.def.return_visibility == Visibility::Public {
        Some(ResolverError::UnnecessaryPub {
            ident: func.name_ident().clone(),
            position: PubPosition::ReturnType,
        })
    } else {
        None
    }
}

/// Only arguments to entrypoint functions may have a non-private visibility modifier applied to them.
///
/// Other functions are disallowed from declaring the visibility of their arguments.
pub(super) fn unnecessary_pub_argument(
    func: &NoirFunction,
    arg_visibility: Visibility,
    is_entry_point: bool,
) -> Option<ResolverError> {
    if arg_visibility == Visibility::Public && !is_entry_point {
        Some(ResolverError::UnnecessaryPub {
            ident: func.name_ident().clone(),
            position: PubPosition::Parameter,
        })
    } else {
        None
    }
}

/// Check if an assignment is overflowing with respect to `annotated_type`
/// in a declaration statement where `annotated_type` is a signed or unsigned integer
pub(crate) fn overflowing_int(
    interner: &NodeInterner,
    rhs_expr: &ExprId,
    annotated_type: &Type,
) -> Vec<TypeCheckError> {
    let expr = interner.expression(rhs_expr);
    let span = interner.expr_span(rhs_expr);

    let mut errors = Vec::with_capacity(2);
    match expr {
        HirExpression::Literal(HirLiteral::Integer(value, negative)) => match annotated_type {
            Type::Integer(Signedness::Unsigned, bit_count) => {
                let bit_count: u32 = (*bit_count).into();
                let max = 2u128.pow(bit_count) - 1;
                if value > max.into() || negative {
                    errors.push(TypeCheckError::OverflowingAssignment {
                        expr: if negative { -value } else { value },
                        ty: annotated_type.clone(),
                        range: format!("0..={}", max),
                        span,
                    });
                }
            }
            Type::Integer(Signedness::Signed, bit_count) => {
                let bit_count: u32 = (*bit_count).into();
                let min = 2u128.pow(bit_count - 1);
                let max = 2u128.pow(bit_count - 1) - 1;
                if (negative && value > min.into()) || (!negative && value > max.into()) {
                    errors.push(TypeCheckError::OverflowingAssignment {
                        expr: if negative { -value } else { value },
                        ty: annotated_type.clone(),
                        range: format!("-{}..={}", min, max),
                        span,
                    });
                }
            }
            _ => (),
        },
        HirExpression::Prefix(expr) => {
            overflowing_int(interner, &expr.rhs, annotated_type);
            if expr.operator == UnaryOp::Minus {
                errors.push(TypeCheckError::InvalidUnaryOp {
                    kind: "annotated_type".to_string(),
                    span,
                });
            }
        }
        HirExpression::Infix(expr) => {
            errors.extend(overflowing_int(interner, &expr.lhs, annotated_type));
            errors.extend(overflowing_int(interner, &expr.rhs, annotated_type));
        }
        _ => {}
    }

    errors
}
