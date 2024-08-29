impl Module {
    #[builtin(module_is_contract)]
// docs:start:is_contract
    fn is_contract(self) -> bool {}
    // docs:end:is_contract

    #[builtin(module_functions)]
// docs:start:functions
    fn functions(self) -> [FunctionDefinition] {}
    // docs:end:functions

    #[builtin(module_name)]
// docs:start:name
    fn name(self) -> Quoted {}
    // docs:end:name
}
