impl FunctionDefinition {
    #[builtin(function_def_as_typed_expr)]
    // docs:start:as_typed_expr
    pub comptime fn as_typed_expr(self) -> TypedExpr {}
    // docs:end:as_typed_expr

    #[builtin(function_def_body)]
    // docs:start:body
    pub comptime fn body(self) -> Expr {}
    // docs:end:body

    #[builtin(function_def_disable)]
    // docs:start:disable
    pub comptime fn disable(self, error_message: CtString) {}
    // docs:end:disable

    #[builtin(function_def_has_named_attribute)]
    // docs:start:has_named_attribute
    pub comptime fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
    // docs:end:has_named_attribute

    #[builtin(function_def_is_unconstrained)]
    // docs:start:is_unconstrained
    pub comptime fn is_unconstrained(self) -> bool {}
    // docs:end:is_unconstrained

    #[builtin(function_def_module)]
    // docs:start:module
    pub comptime fn module(self) -> Module {}
    // docs:end:module

    #[builtin(function_def_name)]
    // docs:start:name
    pub comptime fn name(self) -> Quoted {}
    // docs:end:name

    #[builtin(function_def_parameters)]
    // docs:start:parameters
    pub comptime fn parameters(self) -> [(Quoted, Type)] {}
    // docs:end:parameters

    #[builtin(function_def_return_type)]
    // docs:start:return_type
    pub comptime fn return_type(self) -> Type {}
    // docs:end:return_type

    #[builtin(function_def_visibility)]
    // docs:start:visibility
    pub comptime fn visibility(self) -> Quoted {}
    // docs:end:visibility
}

impl crate::hash::Hash for FunctionDefinition {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: crate::hash::Hasher,
    {
        state.write(function_def_hash(self))
    }
}

impl crate::cmp::Eq for FunctionDefinition {
    comptime fn eq(self, other: Self) -> bool {
        function_def_eq(self, other)
    }
}

#[builtin(function_def_eq)]
comptime fn function_def_eq(_first: FunctionDefinition, _second: FunctionDefinition) -> bool {}

#[builtin(function_def_hash)]
comptime fn function_def_hash(_function: FunctionDefinition) -> Field {}
