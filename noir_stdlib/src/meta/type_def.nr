impl StructDefinition {
    /// Return a syntactic version of this struct definition as a type.
    /// For example, `as_type(quote { type Foo<A, B> { ... } })` would return `Foo<A, B>`
    #[builtin(struct_def_as_type)]
    fn as_type(self) -> Quoted {}

    /// Return each generic on this struct. The names of these generics are unchanged
    /// so users may need to keep name collisions in mind if this is used directly in a macro.
    #[builtin(struct_def_generics)]
    fn generics(self) -> [Quoted] {}

    /// Returns (name, type) pairs of each field in this struct. Each type is as-is
    /// with any generic arguments unchanged.
    #[builtin(struct_def_fields)]
    fn fields(self) -> [(Quoted, Quoted)] {}
}
