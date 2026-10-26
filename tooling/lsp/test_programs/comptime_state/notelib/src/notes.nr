use macros::{register, with_kind};

#[register]
#[with_kind]
pub struct LibThing {}

impl LibThing {
    pub fn describe(self) {}
}

#[with_kind]
pub struct LibOther {}

pub fn kinds(a: LibThing, b: LibOther) {
    a.kind();
    b.kind()
}
