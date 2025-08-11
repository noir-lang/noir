pub fn foo(_: [_; _]) -> [_; _] {
    [1]
}

pub struct Foo {
    x: [_; _],
}

impl Foo {
    fn foo(_: [_; _]) -> [_; _] {
        [1]
    }
}

pub trait Trait {
    fn trait_method(_: [_; _]) -> [_; _];
}

fn main() {}
