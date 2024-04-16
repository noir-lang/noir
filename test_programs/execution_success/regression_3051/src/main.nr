trait Foo {
    fn foo(self) -> Field;
}

impl Foo for Field {
    fn foo(self) -> Field {
        self + 1
    }
}

fn main() {
    dep::std::println(1.foo());
}
