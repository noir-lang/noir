use crate::{hir::resolution::errors::ResolverError, macros_api::NoirFunction};

pub(crate) fn lint_inlining_attributes(func: &NoirFunction) -> Option<ResolverError> {
    // Inline attributes are only relevant for constrained functions
    // as all unconstrained functions are not inlined and so
    // associated attributes are disallowed.
    let inline_attributes_disallowed = func.def.is_unconstrained;
    if !inline_attributes_disallowed {
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
