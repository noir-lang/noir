struct Foo {}

fn takes_lib_foo(_foo: lib::Foo) {}

pub fn passes_own_foo() {
    takes_lib_foo(Foo {});
}
