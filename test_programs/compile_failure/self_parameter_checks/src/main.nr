pub fn foo(self: i32) {}

pub struct Foo {}

impl Foo {
    fn foo(_x: i32, self) {}
    fn bar(self: i32) {}
    fn baz(self) {}
}

pub trait Trait {
    fn foo(_x: i32, self) {}
    fn bar(self: i32) {}
    fn baz(self) {}
}

fn main() {
    let self = 1;
    let _ = |self| {};
}
