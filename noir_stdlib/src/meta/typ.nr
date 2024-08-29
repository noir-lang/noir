use crate::cmp::Eq;
use crate::option::Option;

impl Type {
    #[builtin(type_as_array)]
// docs:start:as_array
    fn as_array(self) -> Option<(Type, Type)> {}
    // docs:end:as_array

    #[builtin(type_as_constant)]
// docs:start:as_constant
    fn as_constant(self) -> Option<u32> {}
    // docs:end:as_constant

    #[builtin(type_as_integer)]
// docs:start:as_integer
    fn as_integer(self) -> Option<(bool, u8)> {}
    // docs:end:as_integer

    #[builtin(type_as_slice)]
// docs:start:as_slice
    fn as_slice(self) -> Option<Type> {}
    // docs:end:as_slice

    #[builtin(type_as_struct)]
// docs:start:as_struct
    fn as_struct(self) -> Option<(StructDefinition, [Type])> {}
    // docs:end:as_struct

    #[builtin(type_as_tuple)]
// docs:start:as_tuple
    fn as_tuple(self) -> Option<[Type]> {}
    // docs:end:as_tuple

    #[builtin(type_get_trait_impl)]
// docs:start:get_trait_impl
    fn get_trait_impl(self, constraint: TraitConstraint) -> Option<TraitImpl> {}
    // docs:end:get_trait_impl

    #[builtin(type_implements)]
// docs:start:implements
    fn implements(self, constraint: TraitConstraint) -> bool {}
    // docs:end:implements

    #[builtin(type_is_bool)]
// docs:start:is_bool
    fn is_bool(self) -> bool {}
    // docs:end:is_bool

    #[builtin(type_is_field)]
// docs:start:is_field
    fn is_field(self) -> bool {}
    // docs:end:is_field
}

impl Eq for Type {
    fn eq(self, other: Self) -> bool {
        type_eq(self, other)
    }
}

#[builtin(type_eq)]
fn type_eq(_first: Type, _second: Type) -> bool {}
