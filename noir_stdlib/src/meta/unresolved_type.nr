//! Contains methods on the built-in `UnresolvedType` type for the syntax of types.

use crate::option::Option;

impl UnresolvedType {
    /// If this is a mutable reference type `&mut T`, returns the mutable type `T`.
    #[builtin(unresolved_type_as_mutable_reference)]
    // docs:start:as_mutable_reference
    comptime fn as_mutable_reference(self) -> Option<UnresolvedType> {}
    // docs:end:as_mutable_reference

    /// If this is a slice `&[T]`, returns the element type `T`.
    #[builtin(unresolved_type_as_slice)]
    // docs:start:as_slice
    comptime fn as_slice(self) -> Option<UnresolvedType> {}
    // docs:end:as_slice

    /// Returns `true` if this type is `bool`.
    #[builtin(unresolved_type_is_bool)]
    // docs:start:is_bool
    comptime fn is_bool(self) -> bool {}
    // docs:end:is_bool

    /// Returns true if this type refers to the `Field` type.
    #[builtin(unresolved_type_is_field)]
    // docs:start:is_field
    pub comptime fn is_field(self) -> bool {}
    // docs:end:is_field

    /// Returns true if this type is the unit `()` type.
    #[builtin(unresolved_type_is_unit)]
    // docs:start:is_unit
    comptime fn is_unit(self) -> bool {}
    // docs:end:is_unit
}
