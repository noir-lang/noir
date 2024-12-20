trait Foo {
    fn foo(self) -> Field;
}

trait Bar: Foo {
    fn bar(self) -> Field {
        self.foo() + 1
    }

    fn baz(self) -> Field;
}

struct Struct {
    x: Field,
}

impl Foo for Struct {
    fn foo(self) -> Field {
        self.x
    }
}

impl Bar for Struct {
    fn baz(self) -> Field {
        self.foo() + 2
    }
}

fn main() {
    let s = Struct { x: 1 };
    assert_eq(s.bar(), 2);
    assert_eq(s.baz(), 3);
}
