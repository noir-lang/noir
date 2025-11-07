use crate::option::Option;

impl Module {
    #[builtin(module_add_item)]
    // docs:start:add_item
    pub comptime fn add_item(self, item: Quoted) {}
    // docs:end:add_item

    #[builtin(module_has_named_attribute)]
    // docs:start:has_named_attribute
    pub comptime fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
    // docs:end:has_named_attribute

    #[builtin(module_is_contract)]
    // docs:start:is_contract
    pub comptime fn is_contract(self) -> bool {}
    // docs:end:is_contract

    #[builtin(module_functions)]
    // docs:start:functions
    pub comptime fn functions(self) -> [FunctionDefinition] {}
    // docs:end:functions

    #[builtin(module_structs)]
    // docs:start:structs
    pub comptime fn structs(self) -> [TypeDefinition] {}
    // docs:end:structs

    #[builtin(module_child_modules)]
    // docs:start:child_modules
    pub comptime fn child_modules(self) -> [Module] {}
    // docs:end:child_modules

    #[builtin(module_name)]
    // docs:start:name
    pub comptime fn name(self) -> Quoted {}
    // docs:end:name

    #[builtin(module_parent)]
    // docs:start:parent
    pub comptime fn parent(self) -> Option<Module> {}
    // docs:end:parent
}

impl crate::hash::Hash for Module {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: crate::hash::Hasher,
    {
        state.write(module_hash(self))
    }
}

impl crate::cmp::Eq for Module {
    comptime fn eq(self, other: Self) -> bool {
        module_eq(self, other)
    }
}

#[builtin(module_eq)]
comptime fn module_eq(_first: Module, _second: Module) -> bool {}

#[builtin(module_hash)]
comptime fn module_hash(_module: Module) -> Field {}
