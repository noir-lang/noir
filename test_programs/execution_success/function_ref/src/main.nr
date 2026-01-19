fn main(c: bool) -> pub str<3> {
    let mut f = foo;
    if c {
        f = bar;
    }
    f()
}

fn foo() -> str<3> {
    "foo"
}

fn bar() -> str<3> {
    "bar"
}
