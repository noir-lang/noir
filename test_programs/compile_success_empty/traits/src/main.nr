use dep::std;

trait MyDefault {
    fn my_default(x: field, y: field) -> Self;
}

struct Foo {
    bar: field,
    array: [field; 2],
}

impl MyDefault for Foo {
    fn my_default(x: field,y: field) -> Self {
        Self { bar: x, array: [x,y] }
    }
}

fn main(x: field, y: field) {
    let first = Foo::my_default(x, y);
    assert(first.bar == x);
}
