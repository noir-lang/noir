impl StructDefinition {
    #[builtin(struct_def_add_attribute)]
    // docs:start:add_attribute
    pub comptime fn add_attribute<let N: u32>(self, attribute: str<N>) {}
    // docs:end:add_attribute

    #[builtin(struct_def_add_generic)]
    // docs:start:add_generic
    pub comptime fn add_generic<let N: u32>(self, generic_name: str<N>) -> Type {}
    // docs:end:add_generic

    /// Return a syntactic version of this struct definition as a type.
    /// For example, `as_type(quote { type Foo<A, B> { ... } })` would return `Foo<A, B>`
    #[builtin(struct_def_as_type)]
    // docs:start:as_type
    pub comptime fn as_type(self) -> Type {}
    // docs:end:as_type

    #[builtin(struct_def_has_named_attribute)]
    // docs:start:has_named_attribute
    pub comptime fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
    // docs:end:has_named_attribute

    /// Return each generic on this struct.
    #[builtin(struct_def_generics)]
    // docs:start:generics
    pub comptime fn generics(self) -> [Type] {}
    // docs:end:generics

    /// Returns (name, type) pairs of each field in this struct.
    /// Any generic types used in each field type is automatically substituted with the
    /// provided generic arguments.
    #[builtin(struct_def_fields)]
    // docs:start:fields
    pub comptime fn fields(self, generic_args: [Type]) -> [(Quoted, Type)] {}
    // docs:end:fields

    /// Returns (name, type) pairs of each field in this struct. Each type is as-is
    /// with any generic arguments unchanged. Unless the field types are not needed,
    /// users should generally prefer to use `StructDefinition::fields` over this
    /// function if possible.
    #[builtin(struct_def_fields_as_written)]
    // docs:start:fields_as_written
    pub comptime fn fields_as_written(self) -> [(Quoted, Type)] {}
    // docs:end:fields_as_written

    #[builtin(struct_def_module)]
    // docs:start:module
    pub comptime fn module(self) -> Module {}
    // docs:end:module

    #[builtin(struct_def_name)]
    // docs:start:name
    pub comptime fn name(self) -> Quoted {}
    // docs:end:name

    /// Sets the fields of this struct to the given fields list.
    /// All existing fields of the struct will be overridden with the given fields.
    /// Each element of the fields list corresponds to the name and type of a field.
    /// Each name is expected to be a single identifier.
    #[builtin(struct_def_set_fields)]
    // docs:start:set_fields
    pub comptime fn set_fields(self, new_fields: [(Quoted, Type)]) {}
    // docs:end:set_fields
}

impl crate::hash::Hash for StructDefinition {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: crate::hash::Hasher,
    {
        state.write(struct_def_hash(self))
    }
}

impl crate::cmp::Eq for StructDefinition {
    comptime fn eq(self, other: Self) -> bool {
        struct_def_eq(self, other)
    }
}

#[builtin(struct_def_eq)]
comptime fn struct_def_eq(_first: StructDefinition, _second: StructDefinition) -> bool {}

#[builtin(struct_def_hash)]
comptime fn struct_def_hash(_struct: StructDefinition) -> Field {}
