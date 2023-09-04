use dep::std;

trait Default {
    fn default(x: Field, y: Field) -> Self;

    fn method2(x: Field) -> Field {
	x
    }

}

struct Foo {
    bar: Field,
    array: [Field; 2],
}

impl Default for Foo {
    fn default(x: Field,y: Field) -> Self {
        Self { bar: x, array: [x,y] }
    }

    fn method2(x: Field) -> Field {
	x * 3
    }
}

fn main(x: Field) {
    let first = Foo::method2(x);
    assert(first == 3 * x);
}
