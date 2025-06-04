use crate::cmp::Eq;
use crate::hash::{Hash, Hasher};

impl TraitDefinition {
    #[builtin(trait_def_as_trait_constraint)]
    // docs:start:as_trait_constraint
    pub comptime fn as_trait_constraint(_self: Self) -> TraitConstraint {}
    // docs:end:as_trait_constraint
}

impl Eq for TraitDefinition {
    comptime fn eq(self, other: Self) -> bool {
        trait_def_eq(self, other)
    }
}

impl Hash for TraitDefinition {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(trait_def_hash(self));
    }
}

#[builtin(trait_def_eq)]
comptime fn trait_def_eq(_first: TraitDefinition, _second: TraitDefinition) -> bool {}

#[builtin(trait_def_hash)]
comptime fn trait_def_hash(_def: TraitDefinition) -> Field {}
