mod foo {
    mod bar {
        struct Foo {
            field: Field,
        }

        impl Foo {
            fn foo() {}

            fn bar(self) {}

            fn baz() -> Self {
                Self { field: 1 }
            }
        }
    }
}

use foo::bar::Foo;

fn main(x: Field) -> pub Field {
    let foo1 = Foo { field: 1 };
    let foo2 = Foo { field: 2 };
    let Foo { field } = foo1;
    Foo::foo();
    x
}

fn foo(foo: Foo) -> Foo {
    foo
}
