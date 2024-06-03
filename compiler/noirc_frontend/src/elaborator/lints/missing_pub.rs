use crate::{
    hir::resolution::errors::ResolverError,
    macros_api::{NoirFunction, UnresolvedTypeData, Visibility},
};

/// `pub` is required on return types for entry point functions
pub(crate) fn lint_missing_pub(func: &NoirFunction, is_entry_point: bool) -> Option<ResolverError> {
    if is_entry_point
        && func.return_type().typ != UnresolvedTypeData::Unit
        && func.def.return_visibility == Visibility::Private
    {
        Some(ResolverError::NecessaryPub { ident: func.name_ident().clone() })
    } else {
        None
    }
}
