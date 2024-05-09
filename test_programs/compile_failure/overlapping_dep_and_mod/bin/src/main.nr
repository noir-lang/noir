// bin/main.nr

fn main() -> pub Field {
    foo::bar() + foo::baz()
}

mod foo {
    pub(crate) fn bar() -> Field {
        5
    }
}
