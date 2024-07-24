impl Quoted {
    #[builtin(quoted_as_trait_constraint)]
    fn as_trait_constraint(self) -> TraitConstraint {}

    #[builtin(quoted_as_type)]
    fn as_type(self) -> Type {}
}
