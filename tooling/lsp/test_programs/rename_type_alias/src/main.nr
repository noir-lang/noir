mod foo {
    struct Foo {
    }

    type Bar = Foo;

    mod bar {
        struct Baz {

        }
    }
}

use foo::Foo;
use foo::Bar;
use foo::bar;

fn main() {
    let x: Bar = Foo {};
}

fn x(b: Bar) -> Bar {
    b
}
