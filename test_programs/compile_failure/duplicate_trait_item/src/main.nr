// Test that having an associated type and associated constant with the same name
// produces a duplicate definition error
pub trait Foo {
    type Bar;
    let Bar: u32;
}

impl Foo for () {
    type Bar = ();
    let Bar: u32 = 0;
}

fn main() { }
