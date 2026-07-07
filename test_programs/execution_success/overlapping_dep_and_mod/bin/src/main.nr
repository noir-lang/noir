// bin/main.nr

fn main() -> pub Field {
    assert(foo::bar() + ::foo::bar() == 11);
    assert(foo::bar() == 5);
    assert(::foo::bar() == 6);
    foo::bar() + ::foo::bar()
}

mod foo {
    pub(crate) fn bar() -> Field {
        5
    }
}
