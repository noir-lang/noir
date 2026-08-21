use crate::option::Option;

/// A generic parameter of a [`TypeDefinition`], returned from
/// [`TypeDefinition::generics`].
///
/// Fields are private so that new information (e.g. bounds or defaults) can be added
/// without breaking source that reads them.
pub comptime struct TypeGeneric {
    typ: Type,
    numeric_type: Option<Type>,
}

impl TypeGeneric {
    /// The generic's type variable.
    pub comptime fn typ(self) -> Type {
        self.typ
    }

    /// If this generic is numeric, the numeric type; otherwise `None`.
    pub comptime fn numeric_type(self) -> Option<Type> {
        self.numeric_type
    }
}

/// A field of a [`TypeDefinition`], returned from
/// [`TypeDefinition::fields`] and [`TypeDefinition::fields_as_written`].
///
/// Fields are private so that additional information (e.g. the field's location)
/// can be added without breaking source that reads them.
pub comptime struct StructField {
    name: Quoted,
    typ: Type,
    visibility: Quoted,
}

impl StructField {
    /// The field's declared name, as a [`Quoted`] identifier.
    pub comptime fn name(self) -> Quoted {
        self.name
    }

    /// The field's declared type.
    pub comptime fn typ(self) -> Type {
        self.typ
    }

    /// The field's declared visibility, as a [`Quoted`] visibility keyword.
    pub comptime fn visibility(self) -> Quoted {
        self.visibility
    }
}

impl TypeDefinition {
    #[builtin(type_def_add_abi)]
    // docs:start:add_abi
    pub comptime fn add_abi(self, abi_argument: CtString) {}
    // docs:end:add_abi

    /// Return a syntactic version of this type definition as a type.
    /// For example, `type Foo<A, B> { ... })` would return `Foo<A, B>` when called with this method.
    #[builtin(type_def_as_type)]
    // docs:start:as_type
    pub comptime fn as_type(self) -> Type {}
    // docs:end:as_type

    /// Return this type applied to the given generic arguments.
    /// For example, given `type Foo<A, B> { ... })` could be applied to `[quote[i32].as_type(), quote[Field].as_type()].as_vector()`
    /// to return `Foo<i32, Field>`.
    ///
    /// If an incorrect number of generic arguments are given, this function will return None.
    /// Otherwise, this function returns Some with the resulting type inside.
    #[builtin(type_def_as_type_with_generics)]
    // docs:start:as_type_with_generics
    pub comptime fn as_type_with_generics(self, generics: [Type]) -> Option<Type> {}
    // docs:end:as_type_with_generics

    #[builtin(type_def_has_named_attribute)]
    // docs:start:has_named_attribute
    pub comptime fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
    // docs:end:has_named_attribute

    #[builtin(type_def_named_attribute_args)]
    // docs:start:named_attribute_args
    pub comptime fn named_attribute_args<let N: u32>(self, name: str<N>) -> [[Quoted]] {}
    // docs:end:named_attribute_args

    #[builtin(type_def_has_builtin_attribute)]
    // docs:start:has_builtin_attribute
    pub comptime fn has_builtin_attribute<let N: u32>(self, name: str<N>) -> bool {}
    // docs:end:has_builtin_attribute
    #[builtin(type_def_location)]
    // docs:start:location
    pub comptime fn location(self) -> Location {}
    // docs:end:location

    /// Return each generic in this type definition. If a generic is numeric, its
    /// [`TypeGeneric::numeric_type`] holds the numeric type.
    // docs:start:generics
    pub comptime fn generics(self) -> [TypeGeneric] {
        // docs:end:generics
        self.generics_raw().map(|g: (Type, Option<Type>)| TypeGeneric {
            typ: g.0,
            numeric_type: g.1,
        })
    }

    #[builtin(type_def_generics)]
    comptime fn generics_raw(self) -> [(Type, Option<Type>)] {}

    /// Returns each field in this type. Any generic types used in each field type is
    /// automatically substituted with the provided generic arguments.
    // docs:start:fields
    pub comptime fn fields(self, generic_args: [Type]) -> [StructField] {
        // docs:end:fields
        self.fields_raw(generic_args).map(|f: (Quoted, Type, Quoted)| StructField {
            name: f.0,
            typ: f.1,
            visibility: f.2,
        })
    }

    #[builtin(type_def_fields)]
    comptime fn fields_raw(self, generic_args: [Type]) -> [(Quoted, Type, Quoted)] {}

    /// Returns each field in this type. Each type is as-is with any generic arguments
    /// unchanged. Unless the field types are not needed, users should generally prefer
    /// to use [`TypeDefinition::fields`][Self::fields] over this function if possible.
    // docs:start:fields_as_written
    pub comptime fn fields_as_written(self) -> [StructField] {
        // docs:end:fields_as_written
        self.fields_as_written_raw().map(|f: (Quoted, Type, Quoted)| StructField {
            name: f.0,
            typ: f.1,
            visibility: f.2,
        })
    }

    #[builtin(type_def_fields_as_written)]
    comptime fn fields_as_written_raw(self) -> [(Quoted, Type, Quoted)] {}

    #[builtin(type_def_module)]
    // docs:start:module
    pub comptime fn module(self) -> Module {}
    // docs:end:module

    #[builtin(type_def_name)]
    // docs:start:name
    pub comptime fn name(self) -> Quoted {}
    // docs:end:name
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
    #[builtin(type_def_eq)]
    comptime fn eq(self, other: Self) -> bool {}
}

#[builtin(type_def_hash)]
comptime fn type_def_hash(_type: TypeDefinition) -> Field {}
