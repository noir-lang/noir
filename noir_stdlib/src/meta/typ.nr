use crate::cmp::Eq;
use crate::option::Option;

#[builtin(fresh_type_variable)]
// docs:start:fresh_type_variable
pub comptime fn fresh_type_variable() -> Type {}
// docs:end:fresh_type_variable

impl Type {
    #[builtin(type_as_array)]
// docs:start:as_array
    comptime fn as_array(self) -> Option<(Type, Type)> {}
    // docs:end:as_array

    #[builtin(type_as_constant)]
// docs:start:as_constant
    comptime fn as_constant(self) -> Option<u32> {}
    // docs:end:as_constant

    #[builtin(type_as_integer)]
// docs:start:as_integer
    comptime fn as_integer(self) -> Option<(bool, u8)> {}
    // docs:end:as_integer

    #[builtin(type_as_slice)]
// docs:start:as_slice
    comptime fn as_slice(self) -> Option<Type> {}
    // docs:end:as_slice

    #[builtin(type_as_str)]
// docs:start:as_str
    comptime fn as_str(self) -> Option<Type> {}
    // docs:end:as_str

    #[builtin(type_as_struct)]
// docs:start:as_struct
    comptime fn as_struct(self) -> Option<(StructDefinition, [Type])> {}
    // docs:end:as_struct

    #[builtin(type_as_tuple)]
// docs:start:as_tuple
    comptime fn as_tuple(self) -> Option<[Type]> {}
    // docs:end:as_tuple

    #[builtin(type_get_trait_impl)]
// docs:start:get_trait_impl
    comptime fn get_trait_impl(self, constraint: TraitConstraint) -> Option<TraitImpl> {}
    // docs:end:get_trait_impl

    #[builtin(type_implements)]
// docs:start:implements
    comptime fn implements(self, constraint: TraitConstraint) -> bool {}
    // docs:end:implements

    #[builtin(type_is_bool)]
// docs:start:is_bool
    comptime fn is_bool(self) -> bool {}
    // docs:end:is_bool

    #[builtin(type_is_field)]
// docs:start:is_field
    comptime fn is_field(self) -> bool {}
    // docs:end:is_field
}

impl Eq for Type {
    comptime fn eq(self, other: Self) -> bool {
        type_eq(self, other)
    }
}

impl crate::hash::Hash for Type {
    comptime fn hash<H>(self, state: &mut H) where H: crate::hash::Hasher {
        state.write(type_hash(self))
    }
}

#[builtin(type_eq)]
comptime fn type_eq(_first: Type, _second: Type) -> bool {}

#[builtin(type_hash)]
comptime fn type_hash(_typ: Type) -> Field {}
