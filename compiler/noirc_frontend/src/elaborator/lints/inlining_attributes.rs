use crate::{hir::resolution::errors::ResolverError, macros_api::NoirFunction};

/// Inline attributes are only relevant for constrained functions
/// as all unconstrained functions are not inlined and so
/// associated attributes are disallowed.
pub(crate) fn lint_inlining_attributes(func: &NoirFunction) -> Option<ResolverError> {
    if !func.def.is_unconstrained {
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
