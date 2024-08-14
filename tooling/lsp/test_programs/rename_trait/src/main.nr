mod foo {
    mod bar {
        trait Foo {
            fn foo() {}
        }
    }
}

use foo::bar::Foo;

struct Bar {

}

impl Foo for Bar {

}

fn main() {
    foo::bar::Foo::foo();
}
