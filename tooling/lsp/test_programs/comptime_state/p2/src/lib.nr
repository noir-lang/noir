use macros::{expect_registered, register};

#[register]
pub struct ThingTwo {}

// One registration from notelib, one from this package.
#[expect_registered(2)]
pub struct CheckTwo {}

#[derive(Eq)]
pub struct DerivedTwo {
    x: Field,
}
