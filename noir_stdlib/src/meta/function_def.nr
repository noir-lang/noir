impl FunctionDefinition {
    #[builtin(function_def_name)]
    fn name(self) -> Quoted {}

    #[builtin(function_def_parameters)]
    fn parameters(self) -> [(Quoted, Type)] {}

    #[builtin(function_def_return_type)]
    fn return_type(self) -> Type {}

    #[builtin(function_def_set_body)]
    fn set_body(mut self, body: Quoted) {}

    #[builtin(function_def_set_parameters)]
    fn set_parameters(mut self, parameters: [(Quoted, Type)]) {}

    #[builtin(function_def_set_return_type)]
    fn set_return_type(mut self, return_type: Type) {}
}
