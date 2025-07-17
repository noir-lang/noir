type Foo = Field;

fn returns_a_foo() -> Foo {
    42
}

fn main() {
    let _ = returns_a_foo();
}
