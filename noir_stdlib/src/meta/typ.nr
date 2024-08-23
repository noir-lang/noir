use crate::cmp::Eq;
use crate::option::Option;

impl Type {
    #[builtin(type_as_array)]
    fn as_array(self) -> Option<(Type, Type)> {}

    #[builtin(type_as_constant)]
    fn as_constant(self) -> Option<u32> {}

    #[builtin(type_as_integer)]
    fn as_integer(self) -> Option<(bool, u8)> {}

    #[builtin(type_as_slice)]
    fn as_slice(self) -> Option<Type> {}

    #[builtin(type_as_struct)]
    fn as_struct(self) -> Option<(StructDefinition, [Type])> {}

    #[builtin(type_as_tuple)]
    fn as_tuple(self) -> Option<[Type]> {}

    #[builtin(type_get_trait_impl)]
    fn get_trait_impl(self, constraint: TraitConstraint) -> Option<TraitImpl> {}

    #[builtin(type_implements)]
    fn implements(self, constraint: TraitConstraint) -> bool {}

    #[builtin(type_is_bool)]
    fn is_bool(self) -> bool {}

    #[builtin(type_is_field)]
    fn is_field(self) -> bool {}
}

impl Eq for Type {
    fn eq(self, other: Self) -> bool {
        type_eq(self, other)
    }
}

#[builtin(type_eq)]
fn type_eq(_first: Type, _second: Type) -> bool {}
