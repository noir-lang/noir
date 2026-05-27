use crate::cmp::Eq;
use crate::hash::{Hash, Hasher};

impl Eq for Location {
    comptime fn eq(self, other: Self) -> bool {
        location_eq(self, other)
    }
}

impl Hash for Location {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(location_hash(self));
    }
}

#[builtin(location_eq)]
comptime fn location_eq(_first: Location, _second: Location) -> bool {}

#[builtin(location_hash)]
comptime fn location_hash(_loc: Location) -> Field {}
