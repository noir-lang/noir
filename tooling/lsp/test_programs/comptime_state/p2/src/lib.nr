use macros::{expect_registered, register, with_kind};
use notelib::notes::LibThing;

#[register]
#[with_kind]
pub struct ThingTwo {}

pub fn kind_of(t: ThingTwo) {
    t.kind()
}

// One registration from notelib, one from this package.
#[expect_registered(2)]
pub struct CheckTwo {}

#[derive(Eq)]
pub struct DerivedTwo {
    x: Field,
}

pub fn take(_thing: LibThing) {}
