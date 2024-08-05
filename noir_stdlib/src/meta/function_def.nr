impl FunctionDefinition {
    #[builtin(function_def_parameters)]
    fn parameters(self) -> [(Quoted, Type)] {}
}