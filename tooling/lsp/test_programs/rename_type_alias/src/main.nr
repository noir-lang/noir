mod foo {
    struct Foo {
    }

    type Bar = Foo;
}

use foo::Foo;
use foo::Bar;

fn main() {
    let x: Bar = Foo {};
}

fn x(b: Bar) -> Bar {
    b
}
