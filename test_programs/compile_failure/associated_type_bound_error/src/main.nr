trait Foo {
    type E: Bar;
}

trait Bar {
    fn bar_method(self);
}

impl Foo for i32 {
    type E = i32;
}

fn main() {}
