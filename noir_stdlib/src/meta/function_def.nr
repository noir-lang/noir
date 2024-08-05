impl FunctionDefinition {
    #[builtin(function_def_parameters)]
    fn parameters(self) -> [(Quoted, Type)] {}

    #[builtin(function_def_return_type)]
    fn return_type(self) -> Type {}
}
