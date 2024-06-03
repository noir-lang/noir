use crate::{graph::CrateId, hir::resolution::errors::ResolverError, macros_api::NoirFunction};

/// Attempting to define new low level (`#[builtin]` or `#[foreign]`) functions outside of the stdlib is disallowed.
pub(crate) fn lint_low_level_function_outside_stdlib(
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
