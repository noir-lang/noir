use crate::option::Option;

impl TypeDefinition {
    #[builtin(type_def_add_attribute)]
    // docs:start:add_attribute
    pub comptime fn add_attribute<let N: u32>(self, attribute: str<N>) {}
    // docs:end:add_attribute

    #[builtin(type_def_add_generic)]
    // docs:start:add_generic
    pub comptime fn add_generic<let N: u32>(self, generic_name: str<N>) -> Type {}
    // docs:end:add_generic

    /// Return a syntactic version of this type definition as a type.
    /// For example, `as_type(quote { type Foo<A, B> { ... } })` would return `Foo<A, B>`
    #[builtin(type_def_as_type)]
    // docs:start:as_type
    pub comptime fn as_type(self) -> Type {}
    // docs:end:as_type

    #[builtin(type_def_has_named_attribute)]
    // docs:start:has_named_attribute
    pub comptime fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
    // docs:end:has_named_attribute

    /// Return (type, option<type>) pairs of each generic in this type definition.
    /// If a generic is numeric, the second element of the pair will contain the numeric type.
    #[builtin(type_def_generics)]
    // docs:start:generics
    pub comptime fn generics(self) -> [(Type, Option<Type>)] {}
    // docs:end:generics

    /// Returns (name, type, visibility) tuples of each field in this type.
    /// Any generic types used in each field type is automatically substituted with the
    /// provided generic arguments.
    #[builtin(type_def_fields)]
    // docs:start:fields
    pub comptime fn fields(self, generic_args: [Type]) -> [(Quoted, Type, Quoted)] {}
    // docs:end:fields

    /// Returns (name, type, visibility) tuples of each field in this type. Each type is as-is
    /// with any generic arguments unchanged. Unless the field types are not needed,
    /// users should generally prefer to use `TypeDefinition::fields` over this
    /// function if possible.
    #[builtin(type_def_fields_as_written)]
    // docs:start:fields_as_written
    pub comptime fn fields_as_written(self) -> [(Quoted, Type, Quoted)] {}
    // docs:end:fields_as_written

    #[builtin(type_def_module)]
    // docs:start:module
    pub comptime fn module(self) -> Module {}
    // docs:end:module

    #[builtin(type_def_name)]
    // docs:start:name
    pub comptime fn name(self) -> Quoted {}
    // docs:end:name

    /// Sets the fields of this type to the given fields list.
    /// All existing fields of the type will be overridden with the given fields.
    /// Each element of the fields list corresponds to the name and type of a field.
    /// Each name is expected to be a single identifier.
    #[builtin(type_def_set_fields)]
    // docs:start:set_fields
    pub comptime fn set_fields(self, new_fields: [(Quoted, Type, Quoted)]) {}
    // docs:end:set_fields
}

impl crate::hash::Hash for TypeDefinition {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: crate::hash::Hasher,
    {
        state.write(type_def_hash(self))
    }
}

impl crate::cmp::Eq for TypeDefinition {
    comptime fn eq(self, other: Self) -> bool {
        type_def_eq(self, other)
    }
}

#[builtin(type_def_eq)]
comptime fn type_def_eq(_first: TypeDefinition, _second: TypeDefinition) -> bool {}

#[builtin(type_def_hash)]
comptime fn type_def_hash(_type: TypeDefinition) -> Field {}
