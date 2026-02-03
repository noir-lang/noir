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

pub struct Gen<T> {}
pub trait Trait2<T> {}

pub fn bar<T>()
where
    Gen<_>: Trait2<T>,
{}

impl<T> Trait2<T> for Gen<T>
where
    Gen<_>: Trait2<_>,
{}

fn main() {}
