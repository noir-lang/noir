use crate::option::Option;

impl TypedExpr {
    #[builtin(typed_expr_as_function_definition)]
    // docs:start:as_function_definition
    fn as_function_definition(self) -> Option<FunctionDefinition> {}
    // docs:end:as_function_definition
}
