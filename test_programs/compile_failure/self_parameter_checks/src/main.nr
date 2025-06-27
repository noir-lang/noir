pub fn foo(self: i32) {}

pub struct Foo {}

impl Foo {
    fn foo(_x: i32, self) {}
    fn bar(self: i32) {}
    fn baz(self) {}
    fn qux(self: &mut Self) {}
}

pub trait Trait {
    fn foo(_x: i32, self) {}
    fn bar(self: i32) {}
    fn baz(self) {}
    fn qux(self: &mut Self) {}
}

fn main() {
    let self = 1;
    let mut self = 2;
    let _ = |self| {};
}
