mod foo {
    mod bar {
        struct Foo {
            field: Field,
        }
    }
}

use foo::bar::Foo;

fn main(x: Field) -> pub Field {
    let foo1 = Foo { field: 1 };
    let foo2 = Foo { field: 2 };
    let Foo { field } = foo1;
    x
}

fn foo(foo: Foo) {}
