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
/// such as `Type::implements` or `Type::get_trait_impl` to potentially grab some of the types used.
///
/// Note that calling `Type::implements` or `Type::get_trait_impl` on a type variable will always
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

impl Type {
    /// If this type is an array, return a pair of (element type, size type).
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let array_type = quote { [Field; 3] }.as_type();
    ///     let (field_type, three_type) = array_type.as_array().unwrap();
    ///
    ///     assert(field_type.is_field());
    ///     assert_eq(three_type.as_constant().unwrap(), 3);
    /// }
    /// ```
    #[builtin(type_as_array)]
    // docs:start:as_array
    pub comptime fn as_array(self) -> Option<(Type, Type)> {}
    // docs:end:as_array

    /// If this type is a constant integer (such as the `3` in the array type `[Field; 3]`),
    /// return the numeric constant.
    #[builtin(type_as_constant)]
    // docs:start:as_constant
    pub comptime fn as_constant(self) -> Option<u32> {}
    // docs:end:as_constant

    /// If this is an integer type, return a boolean which is `true`
    /// if the type is signed, as well as the number of bits of this integer type.
    #[builtin(type_as_integer)]
    // docs:start:as_integer
    pub comptime fn as_integer(self) -> Option<(bool, u8)> {}
    // docs:end:as_integer

    /// If this is a mutable reference type `&mut T`, returns the mutable type `T`.
    #[builtin(type_as_mutable_reference)]
    // docs:start:as_mutable_reference
    comptime fn as_mutable_reference(self) -> Option<Type> {}
    // docs:end:as_mutable_reference

    /// If this is a slice type, return the element type of the slice.
    #[builtin(type_as_slice)]
    // docs:start:as_slice
    pub comptime fn as_slice(self) -> Option<Type> {}
    // docs:end:as_slice

    /// If this is a `str<N>` type, returns the length `N` as a type.
    #[builtin(type_as_str)]
    // docs:start:as_str
    pub comptime fn as_str(self) -> Option<Type> {}
    // docs:end:as_str

    /// If this is a struct type, returns the struct in addition to any generic arguments on this type.
    #[builtin(type_as_struct)]
    // docs:start:as_struct
    pub comptime fn as_struct(self) -> Option<(StructDefinition, [Type])> {}
    // docs:end:as_struct

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
    comptime fn is_unit(self) -> bool {}
    // docs:end:is_unit
}

impl Eq for Type {
    /// Note that this is syntactic equality, this is not the same as whether two types will type check
    /// to be the same type. Unless type inference or generics are being used however, users should not
    /// typically have to worry about this distinction.
    comptime fn eq(self, other: Self) -> bool {
        type_eq(self, other)
    }
}

impl crate::hash::Hash for Type {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: crate::hash::Hasher,
    {
        state.write(type_hash(self))
    }
}

#[builtin(type_eq)]
comptime fn type_eq(_first: Type, _second: Type) -> bool {}

#[builtin(type_hash)]
comptime fn type_hash(_typ: Type) -> Field {}
