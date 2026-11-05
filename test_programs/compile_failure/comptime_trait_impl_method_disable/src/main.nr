trait Foo {
    fn foo(self);
}

struct Bar {}

impl Foo for Bar {
    fn foo(self) {}
}

#[disable_foo]
fn helper() {}

comptime fn disable_foo(_: FunctionDefinition) {
    let bar = quote { Bar }.as_type();
    let foo = quote { Foo }.as_trait_constraint();
    let trait_impl = bar.get_trait_impl(foo).unwrap();
    trait_impl.methods()[0].disable("foo is disabled");
}

fn main() {
    helper();
    Bar {}.foo();
}
