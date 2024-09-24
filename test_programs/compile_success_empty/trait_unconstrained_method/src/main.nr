trait Foo {
    unconstrained fn identity(self) -> Self {
        self
    }

    unconstrained fn foo(self) -> u64;
}

impl Foo for Field {
    unconstrained fn foo(self) -> u64 {
        self as u64
    }
}

fn main() {
    assert_eq(2.foo() as Field, 2.identity());
}
