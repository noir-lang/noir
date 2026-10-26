use macros::expect_registered;
use notelib::notes::LibThing;

mod things;

// One registration from notelib, one from `things`.
#[expect_registered(2)]
pub struct CheckOne {}

#[derive(Eq)]
pub struct DerivedOne {
    x: Field,
}

pub fn take(_thing: LibThing) {}
