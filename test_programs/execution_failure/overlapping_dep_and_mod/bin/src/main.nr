fn main() -> pub Field {
    assert(foo::bar() + foo::bar() == 3);
    assert(foo::bar() == 1);
    assert(foo::bar() == 2);
    foo::bar() + foo::bar()
}

mod foo {
    pub(crate) fn bar() -> Field {
        1
    }
}
