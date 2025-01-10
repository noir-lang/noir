impl TraitImpl {
    #[builtin(trait_impl_trait_generic_args)]
    // docs:start:trait_generic_args
    pub comptime fn trait_generic_args(self) -> [Type] {}
    // docs:end:trait_generic_args

    #[builtin(trait_impl_methods)]
    // docs:start:methods
    pub comptime fn methods(self) -> [FunctionDefinition] {}
    // docs:end:methods
}
