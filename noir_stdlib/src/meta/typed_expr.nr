//! Contains methods on the built-in `TypedExpr` type for resolved and type-checked expressions.
use crate::option::Option;

impl TypedExpr {
    /// If this expression refers to a function definitions, returns it. Otherwise returns `Option::none()`.
    #[builtin(typed_expr_as_function_definition)]
    // docs:start:as_function_definition
    pub comptime fn as_function_definition(self) -> Option<FunctionDefinition> {}
    // docs:end:as_function_definition

    /// Returns the type of the expression, if the expression could be resolved without errors.
    #[builtin(typed_expr_get_type)]
    // docs:start:get_type
    pub comptime fn get_type(self) -> Option<Type> {}
    // docs:end:get_type
}
