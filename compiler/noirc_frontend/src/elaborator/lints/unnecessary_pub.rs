use crate::{
    hir::resolution::errors::{PubPosition, ResolverError},
    macros_api::{NoirFunction, Visibility},
};

/// Only entrypoint functions require a `pub` visibility modifier applied to their return types.
///
/// Application of `pub` to other functions is not meaningful and is a mistake.
pub(crate) fn lint_unnecessary_pub_return(
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

pub(crate) fn lint_unnecessary_pub_argument(
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
