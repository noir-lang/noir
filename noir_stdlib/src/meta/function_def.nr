impl FunctionDefinition {
    #[builtin(function_def_add_attribute)]
    // docs:start:add_attribute
    comptime fn add_attribute<let N: u32>(self, attribute: str<N>) {}
    // docs:end:add_attribute

    #[builtin(function_def_body)]
    // docs:start:body
    comptime fn body(self) -> Expr {}
    // docs:end:body

    #[builtin(function_def_has_named_attribute)]
    // docs:start:has_named_attribute
    comptime fn has_named_attribute(self, name: Quoted) -> bool {}
    // docs:end:has_named_attribute

    #[builtin(function_def_is_unconstrained)]
    // docs:start:is_unconstrained
    comptime fn is_unconstrained(self) -> bool {}
    // docs:end:is_unconstrained

    #[builtin(function_def_module)]
    // docs:start:module
    comptime fn module(self) -> Module {}
    // docs:end:module

    #[builtin(function_def_name)]
    // docs:start:name
    comptime fn name(self) -> Quoted {}
    // docs:end:name

    #[builtin(function_def_parameters)]
    // docs:start:parameters
    comptime fn parameters(self) -> [(Quoted, Type)] {}
    // docs:end:parameters

    #[builtin(function_def_return_type)]
    // docs:start:return_type
    comptime fn return_type(self) -> Type {}
    // docs:end:return_type

    #[builtin(function_def_set_body)]
    // docs:start:set_body
    comptime fn set_body(self, body: Expr) {}
    // docs:end:set_body

    #[builtin(function_def_set_parameters)]
    // docs:start:set_parameters
    comptime fn set_parameters(self, parameters: [(Quoted, Type)]) {}
    // docs:end:set_parameters

    #[builtin(function_def_set_return_type)]
    // docs:start:set_return_type
    comptime fn set_return_type(self, return_type: Type) {}
    // docs:end:set_return_type

    #[builtin(function_def_set_return_public)]
    // docs:start:set_return_public
    comptime fn set_return_public(self, public: bool) {}
    // docs:end:set_return_public

    #[builtin(function_def_set_unconstrained)]
    // docs:start:set_unconstrained
    comptime fn set_unconstrained(self, value: bool) {}
    // docs:end:set_unconstrained
}
