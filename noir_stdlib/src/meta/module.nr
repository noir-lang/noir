impl Module {
    #[builtin(module_is_contract)]
    fn is_contract(self) -> bool {}

    #[builtin(module_functions)]
    fn functions(self) -> [FunctionDefinition] {}

    #[builtin(module_name)]
    fn name(self) -> Quoted {}
}
