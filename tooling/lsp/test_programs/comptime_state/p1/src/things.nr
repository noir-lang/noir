use macros::{register, with_kind};

#[register]
#[with_kind]
pub struct ThingOne {}

pub fn kind_of(t: ThingOne) {
    t.kind()
}
