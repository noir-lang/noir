use macros::{expect_registered, register};

#[register]
pub struct ThingOne {}

// One registration from notelib, one from this package.
#[expect_registered(2)]
pub struct CheckOne {}

#[derive(Eq)]
pub struct DerivedOne {
    x: Field,
}

pub fn take(_thing: notelib::LibThing) {}
