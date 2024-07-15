use crate::hash::{Hash, Hasher};
use crate::cmp::Eq;

impl Eq for TraitConstraint {
    fn eq(self, other: Self) -> bool {
        constraint_eq(self, other)
    }
}

impl Hash for TraitConstraint {
    fn hash<H>(self, state: &mut H) where H: Hasher {
        state.write(constraint_hash(self));
    }
}

#[builtin(trait_constraint_eq)]
fn constraint_eq(_first: TraitConstraint, _second: TraitConstraint) -> bool {}

#[builtin(trait_constraint_hash)]
fn constraint_hash(_constraint: TraitConstraint) -> Field {}
