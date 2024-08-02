use crate::cmp::Eq;
use crate::option::Option;

impl Type {
    #[builtin(type_as_integer)]
    fn as_integer(self) -> Option<(bool, u8)> {}

    #[builtin(type_as_tuple)]
    fn as_tuple(self) -> Option<[Type]> {}

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
