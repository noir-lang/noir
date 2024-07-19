impl TraitDefinition {
    #[builtin(trait_def_as_trait_constraint)]
    fn as_trait_constraint(_self: Self) -> TraitConstraint {}
}
