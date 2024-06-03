use crate::{hir::resolution::errors::ResolverError, macros_api::NoirFunction};

/// `pub` is required on return types for entry point functions
pub(crate) fn lint_test_function_with_args(func: &NoirFunction) -> Option<ResolverError> {
    if func.attributes().is_test_function() && !func.parameters().is_empty() {
        Some(ResolverError::TestFunctionHasParameters { span: func.name_ident().span() })
    } else {
        None
    }
}
