use crate::cmp::Eq;
use crate::hash::{Hash, Hasher};

impl Eq for TraitConstraint {
    comptime fn eq(self, other: Self) -> bool {
        constraint_eq(self, other)
    }
}

impl Hash for TraitConstraint {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(constraint_hash(self));
    }
}

#[builtin(trait_constraint_eq)]
comptime fn constraint_eq(_first: TraitConstraint, _second: TraitConstraint) -> bool {}

#[builtin(trait_constraint_hash)]
comptime fn constraint_hash(_constraint: TraitConstraint) -> Field {}
