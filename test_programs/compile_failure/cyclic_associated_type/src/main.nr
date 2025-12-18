trait Foo {
    type Bar;
}

impl Foo for () {
    // Self::Bar refers to itself, creating a cyclic type
    type Bar = Self::Bar;
}

fn main() { }
