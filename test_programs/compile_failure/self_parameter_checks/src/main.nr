pub fn one(self: i32) {}

pub struct Foo {}

impl Foo {
    fn one(_x: i32, self) {}
    fn two(self: i32) {}
    fn three(self) {}
    fn four(self: &mut Self) {}
    fn five((_x, self): (i32, Self)) {}
}

pub trait Trait {
    fn one(_x: i32, self);
    fn two(self: i32);
    fn three(self);
    fn four(self: &mut Self);
    fn five((_x, self): (i32, Self));
}

impl Trait for Foo {
    fn one(_x: i32, self) {}
    fn two(self: i32) {}
    fn three(self) {}
    fn four(self: &mut Self) {}
    fn five((_x, self): (i32, Self)) {}
}

fn main() {
    let self = 1;
    let mut self = 2;
    let _ = |self| {};
}
