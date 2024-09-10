use crate::option::Option;

impl TypedExpr {
    #[builtin(typed_expr_as_function_definition)]
    // docs:start:as_function_definition
    comptime fn as_function_definition(self) -> Option<FunctionDefinition> {}
    // docs:end:as_function_definition

    #[builtin(typed_expr_get_type)]
    // docs:start:get_type
    comptime fn get_type(self) -> Option<Type> {}
    // docs:end:get_type
}
