trait Foo {
    fn foo(self);
}

struct Bar {}

impl Foo for Bar {
    #[deprecated("has no effect")]
    fn foo(self) {}
}

#[disable_foo]
fn helper() {}

comptime fn disable_foo(_: FunctionDefinition) {
    let bar = quote { Bar }.as_type();
    let foo = quote { Foo }.as_trait_constraint();
    let trait_impl = bar.get_trait_impl(foo).unwrap();
    let method = trait_impl.methods()[0];

    // `#[deprecated]` has no effect on a method in a trait impl, so comptime code can't see it.
    assert(!method.has_named_attribute("deprecated"));

    method.disable("foo is disabled");
}

fn main() {
    helper();
    Bar {}.foo();
}
