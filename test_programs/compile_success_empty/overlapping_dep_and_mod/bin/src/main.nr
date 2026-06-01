fn main() {
    let _ = foo::bar() + ::foo::baz();
}

mod foo {
    pub(crate) fn bar() -> Field {
        5
    }
}
