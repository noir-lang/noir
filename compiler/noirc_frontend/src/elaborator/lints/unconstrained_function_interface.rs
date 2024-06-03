use noirc_errors::Span;

use crate::{hir::type_check::TypeCheckError, node_interner::ExprId, Type};

/// Check that we are not passing a mutable reference from a constrained runtime to an unconstrained runtime.
pub(crate) fn lint_unconstrained_function_args(
    function_args: &[(Type, ExprId, Span)],
) -> Vec<TypeCheckError> {
    function_args
        .iter()
        .filter_map(|(typ, _, span)| {
            if type_contains_mutable_reference(&typ) {
                Some(TypeCheckError::ConstrainedReferenceToUnconstrained { span: span.clone() })
            } else {
                None
            }
        })
        .collect()
}

/// Check that we are not passing a slice from an unconstrained runtime to a constrained runtime.
pub(crate) fn lint_unconstrained_function_return(
    return_type: &Type,
    span: Span,
) -> Option<TypeCheckError> {
    if return_type.contains_slice() {
        Some(TypeCheckError::UnconstrainedSliceReturnToConstrained { span })
    } else if type_contains_mutable_reference(&return_type) {
        Some(TypeCheckError::UnconstrainedReferenceToConstrained { span })
    } else {
        None
    }
}

fn type_contains_mutable_reference(typ: &Type) -> bool {
    matches!(&typ.follow_bindings(), Type::MutableReference(_))
}
