impl UnresolvedType {
    #[builtin(unresolved_type_is_bool)]
    // docs:start:is_bool
    comptime fn is_bool(self) -> bool {}
    // docs:end:is_bool

    #[builtin(unresolved_type_is_field)]
    // docs:start:is_field
    comptime fn is_field(self) -> bool {}
    // docs:end:is_field
}
