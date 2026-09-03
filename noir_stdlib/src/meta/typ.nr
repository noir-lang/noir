//! Contains methods on the built-in `Type` type used for representing a type in the source program.

use crate::cmp::Eq;
use crate::option::Option;

/// Creates and returns an unbound type variable. This is a special kind of type internal
/// to type checking which will type check with any other type. When it is type checked
/// against another type it will also be set to that type. For example, if `a` is a type
/// variable and we have the type equality `(a, i32) = (u8, i32)`, the compiler will set
/// `a` equal to `u8`.
///
/// Unbound type variables will often be rendered as `_` while printing them. Bound type
/// variables will appear as the type they are bound to.
///
/// This can be used in conjunction with functions which internally perform type checks
/// such as [`Type::implements`] or [`Type::get_trait_impl`] to potentially grab some of the types used.
///
/// Note that calling [`Type::implements`] or [`Type::get_trait_impl`] on a type variable will always
/// fail.
///
/// Example:
///
/// ```noir
/// trait Serialize<let N: u32> {}
///
/// impl Serialize<1> for Field {}
///
/// impl<T, let N: u32, let M: u32> Serialize<N * M> for [T; N]
///     where T: Serialize<M> {}
///
/// impl<T, U, let N: u32, let M: u32> Serialize<N + M> for (T, U)
///     where T: Serialize<N>, U: Serialize<M> {}
///
/// fn fresh_variable_example() {
///     let typevar1 = std::meta::typ::fresh_type_variable();
///     let constraint = quote { Serialize<$typevar1> }.as_trait_constraint();
///     let field_type = quote { Field }.as_type();
///
///     // Search for a trait impl (binding typevar1 to 1 when the impl is found):
///     assert(field_type.implements(constraint));
///
///     // typevar1 should be bound to the "1" generic now:
///     assert_eq(typevar1.as_constant().unwrap(), 1);
///
///     // If we want to do the same with a different type, we need to
///     // create a new type variable now that `typevar1` is bound
///     let typevar2 = std::meta::typ::fresh_type_variable();
///     let constraint = quote { Serialize<$typevar2> }.as_trait_constraint();
///     let array_type = quote { [(Field, Field); 5] }.as_type();
///     assert(array_type.implements(constraint));
///
///     // Now typevar2 should be bound to the serialized pair size 2 times the array length 5
///     assert_eq(typevar2.as_constant().unwrap(), 10);
/// }
/// ```
#[builtin(fresh_type_variable)]
// docs:start:fresh_type_variable
pub comptime fn fresh_type_variable() -> Type {}
// docs:end:fresh_type_variable

/// An array type decomposed by [`Type::as_array`] into its element and length
/// components.
pub comptime struct ArrayType {
    element_type: Type,
    length_type: Type,
}

impl ArrayType {
    /// The type of each element the array holds.
    pub comptime fn element_type(self) -> Type {
        self.element_type
    }

    /// The type describing the array's length (a numeric-constant type).
    pub comptime fn length_type(self) -> Type {
        self.length_type
    }
}

/// An integer type decomposed by [`Type::as_integer`] into its signedness and
/// bit width.
pub comptime struct IntegerType {
    is_signed: bool,
    bits: u8,
}

impl IntegerType {
    /// `true` if the integer type is signed.
    pub comptime fn is_signed(self) -> bool {
        self.is_signed
    }

    /// The bit width of the integer type.
    pub comptime fn bits(self) -> u8 {
        self.bits
    }
}

/// A struct or enum type decomposed by [`Type::as_data_type`] into its
/// [`TypeDefinition`] and its concrete generic arguments.
pub comptime struct DataType {
    definition: TypeDefinition,
    generics: [Type],
}

impl DataType {
    /// The [`TypeDefinition`] for this type.
    pub comptime fn definition(self) -> TypeDefinition {
        self.definition
    }

    /// The concrete generic arguments applied to the definition, in source order.
    pub comptime fn generics(self) -> [Type] {
        self.generics
    }
}

impl Type {
    /// If this type is an array, return its element and length types.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let array_type = quote { [Field; 3] }.as_type();
    ///     let array = array_type.as_array().unwrap();
    ///
    ///     assert(array.element_type().is_field());
    ///     assert_eq(array.length_type().as_constant().unwrap(), 3);
    /// }
    /// ```
    // docs:start:as_array
    pub comptime fn as_array(self) -> Option<ArrayType> {
        // docs:end:as_array
        self.as_array_raw().map(|pair: (Type, Type)| {
            ArrayType { element_type: pair.0, length_type: pair.1 }
        })
    }

    #[builtin(type_as_array)]
    comptime fn as_array_raw(self) -> Option<(Type, Type)> {}

    /// If this type is a constant integer (such as the `3` in the array type `[Field; 3]`),
    /// return the numeric constant.
    #[builtin(type_as_constant)]
    // docs:start:as_constant
    pub comptime fn as_constant(self) -> Option<u32> {}
    // docs:end:as_constant

    /// If this is an integer type, return its signedness and bit width.
    // docs:start:as_integer
    pub comptime fn as_integer(self) -> Option<IntegerType> {
        // docs:end:as_integer
        self.as_integer_raw().map(|pair: (bool, u8)| IntegerType { is_signed: pair.0, bits: pair.1 })
    }

    #[builtin(type_as_integer)]
    comptime fn as_integer_raw(self) -> Option<(bool, u8)> {}

    /// If this is a mutable reference type `&mut T`, returns the mutable type `T`.
    #[builtin(type_as_mutable_reference)]
    // docs:start:as_mutable_reference
    pub comptime fn as_mutable_reference(self) -> Option<Type> {}
    // docs:end:as_mutable_reference

    /// If this is a vector type, return the element type of the vector.
    #[builtin(type_as_vector)]
    // docs:start:as_vector
    pub comptime fn as_vector(self) -> Option<Type> {}
    // docs:end:as_vector

    /// If this is a `str<N>` type, returns the length `N` as a type.
    #[builtin(type_as_str)]
    // docs:start:as_str
    pub comptime fn as_str(self) -> Option<Type> {}
    // docs:end:as_str

    #[deprecated("This method has been renamed to `as_data_type`")]
    pub comptime fn as_struct(self) -> Option<DataType> {
        self.as_data_type()
    }

    /// If this is a struct or enum type, returns its [`TypeDefinition`] and
    /// generic arguments.
    // docs:start:as_data_type
    pub comptime fn as_data_type(self) -> Option<DataType> {
        // docs:end:as_data_type
        self.as_data_type_raw().map(|pair: (TypeDefinition, [Type])| {
            DataType { definition: pair.0, generics: pair.1 }
        })
    }

    #[builtin(type_as_data_type)]
    comptime fn as_data_type_raw(self) -> Option<(TypeDefinition, [Type])> {}

    /// If this is a tuple type, returns each element type of the tuple.
    #[builtin(type_as_tuple)]
    // docs:start:as_tuple
    pub comptime fn as_tuple(self) -> Option<[Type]> {}
    // docs:end:as_tuple

    /// Retrieves the trait implementation that implements the given
    /// trait constraint for this type. If the trait constraint is not
    /// found, `None` is returned. Note that since the concrete trait implementation
    /// for a trait constraint specified from a `where` clause is unknown,
    /// this function will return `None` in these cases. If you only want to know
    /// whether a type implements a trait, use `implements` instead.
    ///
    /// Example:
    ///
    /// ```rust
    /// comptime {
    ///     let field_type = quote { Field }.as_type();
    ///     let default = quote { Default }.as_trait_constraint();
    ///
    ///     let the_impl: TraitImpl = field_type.get_trait_impl(default).unwrap();
    ///     assert(the_impl.methods().len(), 1);
    /// }
    /// ```
    #[builtin(type_get_trait_impl)]
    // docs:start:get_trait_impl
    pub comptime fn get_trait_impl(self, constraint: TraitConstraint) -> Option<TraitImpl> {}
    // docs:end:get_trait_impl

    /// Returns `true` if this type implements the given trait. Note that unlike
    /// `get_trait_impl` this will also return true for any `where` constraints
    /// in scope.
    ///
    /// Example:
    ///
    /// ```rust
    /// fn foo<T>() where T: Default {
    ///     comptime {
    ///         let field_type = quote { Field }.as_type();
    ///         let default = quote { Default }.as_trait_constraint();
    ///         assert(field_type.implements(default));
    ///
    ///         let t = quote { T }.as_type();
    ///         assert(t.implements(default));
    ///     }
    /// }
    /// ```
    #[builtin(type_implements)]
    // docs:start:implements
    pub comptime fn implements(self, constraint: TraitConstraint) -> bool {}
    // docs:end:implements

    /// Returns `true` if this type is `bool`.
    #[builtin(type_is_bool)]
    // docs:start:is_bool
    pub comptime fn is_bool(self) -> bool {}
    // docs:end:is_bool

    /// Returns `true` if this type is `Field`.
    #[builtin(type_is_field)]
    // docs:start:is_field
    pub comptime fn is_field(self) -> bool {}
    // docs:end:is_field

    /// Returns `true` if this type is the unit `()` type.
    #[builtin(type_is_unit)]
    // docs:start:is_unit
    pub comptime fn is_unit(self) -> bool {}
    // docs:end:is_unit
}

impl Eq for Type {
    /// Note that this is syntactic equality, this is not the same as whether two types will type check
    /// to be the same type. Unless type inference or generics are being used however, users should not
    /// typically have to worry about this distinction.
    #[builtin(type_eq)]
    comptime fn eq(self, other: Self) -> bool {}
}

impl crate::hash::Hash for Type {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: crate::hash::Hasher,
    {
        state.write(type_hash(self))
    }
}

#[builtin(type_hash)]
comptime fn type_hash(_typ: Type) -> Field {}
