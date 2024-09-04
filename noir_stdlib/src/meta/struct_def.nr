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

    /// Sets the fields of this struct to the given fields list.
    /// All existing fields of the struct will be overridden with the given fields.
    /// Each element of the fields list corresponds to the name and type of a field.
    /// Each name is expected to be a single identifier.
    #[builtin(struct_def_set_fields)]
    // docs:start:set_fields
    fn set_fields(self, new_fields: [(Quoted, Type)]) {}
    // docs:end:set_fields
}
