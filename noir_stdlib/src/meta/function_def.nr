impl FunctionDefinition {
    #[builtin(function_def_name)]
    // docs:start:name
    fn name(self) -> Quoted {}
    // docs:end:name

    #[builtin(function_def_parameters)]
    // docs:start:parameters
    fn parameters(self) -> [(Quoted, Type)] {}
    // docs:end:parameters

    #[builtin(function_def_return_type)]
    // docs:start:return_type
    fn return_type(self) -> Type {}
    // docs:end:return_type

    #[builtin(function_def_set_body)]
    // docs:start:set_body
    fn set_body(self, body: Quoted) {}
    // docs:end:set_body

    #[builtin(function_def_set_parameters)]
    // docs:start:set_parameters
    fn set_parameters(self, parameters: [(Quoted, Type)]) {}
    // docs:end:set_parameters

    #[builtin(function_def_set_return_type)]
    // docs:start:set_return_type
    fn set_return_type(self, return_type: Type) {}
    // docs:end:set_return_type
}
