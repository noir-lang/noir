use crate::cmp::Eq;

impl Eq for Type {
    fn eq(self, other: Self) -> bool {
        type_eq(self, other)
    }
}

#[builtin(type_eq)]
fn type_eq(_first: Type, _second: Type) -> bool {}
