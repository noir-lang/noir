trait Foo {
    fn foo(self) -> Field;
}

impl Foo for Field {
    fn foo(self) -> Field {
        self + 1
    }
}

trait Bar {
    fn bar(self) -> u64;
}

impl Bar for u64 {
    fn bar(self) -> u64 {
        self + 1
    }
}

fn main() {
    std::println(1.foo());
    std::println(1.bar());
}
