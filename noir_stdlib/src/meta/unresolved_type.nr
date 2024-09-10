use crate::option::Option;

impl UnresolvedType {
    #[builtin(unresolved_type_as_mutable_reference)]
    // docs:start:as_mutable_reference
    comptime fn as_mutable_reference(self) -> Option<UnresolvedType> {}
    // docs:end:as_mutable_reference

    #[builtin(unresolved_type_is_bool)]
    // docs:start:is_bool
    comptime fn is_bool(self) -> bool {}
    // docs:end:is_bool

    #[builtin(unresolved_type_is_field)]
    // docs:start:is_field
    comptime fn is_field(self) -> bool {}
    // docs:end:is_field

    #[builtin(unresolved_type_is_unit)]
    // docs:start:is_unit
    comptime fn is_unit(self) -> bool {}
    // docs:end:is_unit
}
