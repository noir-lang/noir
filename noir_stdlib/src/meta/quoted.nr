use crate::cmp::Eq;

impl Quoted {
    #[builtin(quoted_as_trait_constraint)]
    fn as_trait_constraint(self) -> TraitConstraint {}

    #[builtin(quoted_as_type)]
    fn as_type(self) -> Type {}
}

impl Eq for Quoted {
    fn eq(self, other: Quoted) -> bool {
        quoted_eq(self, other)
    }
}

#[builtin(quoted_eq)]
fn quoted_eq(_first: Quoted, _second: Quoted) -> bool {}

