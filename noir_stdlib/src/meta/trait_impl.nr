impl TraitImpl {
    #[builtin(trait_impl_trait_generic_args)]
    fn trait_generic_args(self) -> [Type] {}

    #[builtin(trait_impl_methods)]
    fn methods(self) -> [FunctionDefinition] {}
}
