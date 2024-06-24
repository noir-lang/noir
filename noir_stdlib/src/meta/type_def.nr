impl TypeDefinition {
    /// Return a syntactic version of this type definition as a type.
    /// For example, `as_type(quote { type Foo<A, B> { ... } })` would return `Foo<A, B>`
    #[builtin(type_def_as_type)]
    fn as_type(self) -> Quoted {}

    /// Return each generic on this type. The names of these generics are unchanged
    /// so users may need to keep name collisions in mind if this is used directly in a macro.
    #[builtin(type_def_generics)]
    fn generics(self) -> [Quoted] {}

    /// Returns (name, type) pairs of each field in this type. Each type is as-is
    /// with any generic arguments unchanged.
    #[builtin(type_def_fields)]
    fn fields(self) -> [(Quoted, Quoted)] {}
}
