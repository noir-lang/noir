use macros::expect_registered;
pub use notelib::notes::LibThing;

mod things;

// One registration from notelib, one from `things`.
#[expect_registered(2)]
pub struct CheckOne {}

#[derive(Eq)]
pub struct DerivedOne {
    x: Field,
}

pub fn take(_thing: LibThing) {}

pub fn lib_kind(t: LibThing) {
    t.kind()
}
