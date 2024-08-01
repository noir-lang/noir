use crate::cmp::Eq;

impl Type {
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
