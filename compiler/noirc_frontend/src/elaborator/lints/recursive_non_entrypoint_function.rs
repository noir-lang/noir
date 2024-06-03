use crate::{ast::FunctionKind, hir::resolution::errors::ResolverError, macros_api::NoirFunction};

/// `#[recursive]` attribute is only allowed for entry point functions
pub(crate) fn lint_recursive_non_entrypoint_function(
    func: &NoirFunction,
    is_entry_point: bool,
) -> Option<ResolverError> {
    if !is_entry_point && func.kind == FunctionKind::Recursive {
        Some(ResolverError::MisplacedRecursiveAttribute { ident: func.name_ident().clone() })
    } else {
        None
    }
}
