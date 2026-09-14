use crate::cmp::Eq;
use crate::hash::{Hash, Hasher};

impl Eq for TraitConstraint {
    #[builtin(trait_constraint_eq)]
    comptime fn eq(self, other: Self) -> bool {}
}

impl Hash for TraitConstraint {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(constraint_hash(self));
    }
}

#[builtin(trait_constraint_hash)]
comptime fn constraint_hash(_constraint: TraitConstraint) -> Field {}
