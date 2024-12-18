struct Bar {
    value: Field,
}

struct Foo{
    bar: &mut Bar,
}

impl Foo {
    unconstrained fn crash_fn(self) {}
}

fn main() {
    let foo = Foo { bar: &mut Bar { value: 0 } };

    unsafe {
        //@safety: testing context
        foo.crash_fn()
    };
}
