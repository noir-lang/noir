use dep::std;

trait MyDefault {
    fn my_default(x: field, y: field) -> Self;

    fn method2(x: field) -> field {
	    x
    }
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

fn main(x: field) {
    let first = Foo::method2(x);
    assert(first == x);
}
