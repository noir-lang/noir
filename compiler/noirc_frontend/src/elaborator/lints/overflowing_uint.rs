use crate::{
    ast::UnaryOp,
    hir::type_check::TypeCheckError,
    macros_api::{HirExpression, HirLiteral, NodeInterner},
    node_interner::ExprId,
    Type,
};
use acvm::AcirField;

/// Check if an assignment is overflowing with respect to `annotated_type`
/// in a declaration statement where `annotated_type` is an unsigned integer
pub(crate) fn lint_overflowing_uint(
    interner: &NodeInterner,
    rhs_expr: &ExprId,
    annotated_type: &Type,
) -> Vec<TypeCheckError> {
    let expr = interner.expression(rhs_expr);
    let span = interner.expr_span(rhs_expr);

    let mut errors = Vec::with_capacity(2);
    match expr {
        HirExpression::Literal(HirLiteral::Integer(value, false)) => {
            let v = value.to_u128();
            if let Type::Integer(_, bit_count) = annotated_type {
                let bit_count: u32 = (*bit_count).into();
                let max = 1 << bit_count;
                if v >= max {
                    errors.push(TypeCheckError::OverflowingAssignment {
                        expr: value,
                        ty: annotated_type.clone(),
                        range: format!("0..={}", max - 1),
                        span,
                    });
                };
            };
        }
        HirExpression::Prefix(expr) => {
            lint_overflowing_uint(interner, &expr.rhs, annotated_type);
            if expr.operator == UnaryOp::Minus {
                errors.push(TypeCheckError::InvalidUnaryOp {
                    kind: "annotated_type".to_string(),
                    span,
                });
            }
        }
        HirExpression::Infix(expr) => {
            errors.extend(lint_overflowing_uint(interner, &expr.lhs, annotated_type));
            errors.extend(lint_overflowing_uint(interner, &expr.rhs, annotated_type));
        }
        _ => {}
    }

    errors
}
