impl Module {
    #[builtin(module_add_item)]
    // docs:start:add_item
    fn add_item(self, item: Quoted) {}
    // docs:end:add_item

    #[builtin(module_has_named_attribute)]
    // docs:start:has_named_attribute
    fn has_named_attribute(self, name: Quoted) -> bool {}
    // docs:end:has_named_attribute

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
