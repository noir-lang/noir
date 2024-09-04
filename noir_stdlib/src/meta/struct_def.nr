impl StructDefinition {
    /// Return a syntactic version of this struct definition as a type.
    /// For example, `as_type(quote { type Foo<A, B> { ... } })` would return `Foo<A, B>`
    #[builtin(struct_def_as_type)]
// docs:start:as_type
    fn as_type(self) -> Type {}
    // docs:end:as_type

    /// Return each generic on this struct.
    #[builtin(struct_def_generics)]
// docs:start:generics
    fn generics(self) -> [Type] {}
    // docs:end:generics

    /// Returns (name, type) pairs of each field in this struct. Each type is as-is
    /// with any generic arguments unchanged.
    #[builtin(struct_def_fields)]
// docs:start:fields
    fn fields(self) -> [(Quoted, Type)] {}
    // docs:end:fields
}
