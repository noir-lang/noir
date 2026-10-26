use macros::register;

#[register]
pub struct LibThing {}

impl LibThing {
    pub fn describe(self) {}
}
