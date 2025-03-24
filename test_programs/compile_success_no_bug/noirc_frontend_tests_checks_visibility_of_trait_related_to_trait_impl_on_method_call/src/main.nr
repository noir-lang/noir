mod moo {
    pub struct Bar {}
}

trait Foo {
    fn foo(self);
}

impl Foo for moo::Bar {
    fn foo(self) {}
}

fn main() {
    let bar = moo::Bar {};
    bar.foo();
}
