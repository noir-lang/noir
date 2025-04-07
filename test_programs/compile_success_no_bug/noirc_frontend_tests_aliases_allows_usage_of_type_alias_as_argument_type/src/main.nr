type Foo = Field;

fn accepts_a_foo(x: Foo) {
    assert_eq(x, 42);
}

fn main() {
    accepts_a_foo(42);
}
