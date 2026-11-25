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
    let method = quote { <Bar as Foo>::foo }.as_expr().unwrap().resolve(Option::none());
    method.as_function_definition().unwrap().disable("foo is disabled");
}

fn main() {
    helper();
    // Disabling a trait method covers calls that resolve to an impl of it on a concrete type.
    Bar {}.foo();
    Bar::foo(Bar {});
}
