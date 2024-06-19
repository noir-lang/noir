fn main() {
    let _ = foo::bar() + dep::foo::baz();
}

mod foo {
    pub(crate) fn bar() -> Field {
        5
    }
}
