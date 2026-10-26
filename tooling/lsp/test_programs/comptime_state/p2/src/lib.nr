use macros::{expect_registered, register};
use notelib::notes::LibThing;

#[register]
pub struct ThingTwo {}

// One registration from notelib, one from this package.
#[expect_registered(2)]
pub struct CheckTwo {}

#[derive(Eq)]
pub struct DerivedTwo {
    x: Field,
}

pub fn take(_thing: LibThing) {}
