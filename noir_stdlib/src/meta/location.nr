use crate::cmp::Eq;
use crate::hash::{Hash, Hasher};

impl Eq for Location {
    #[builtin(location_eq)]
    comptime fn eq(self, other: Self) -> bool {}
}

impl Hash for Location {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(location_hash(self));
    }
}

#[builtin(location_hash)]
comptime fn location_hash(_loc: Location) -> Field {}
