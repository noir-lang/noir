use dep::std;

trait Animal {
    fn default(x: Field, y: Field) -> Self;
}

struct Foo {
    bar: Field,
    array: [Field; 2],
}


impl Animal for Foo {
    fn default(x: Field,y: Field) -> Self {
        Self { bar: 0, array: [x,y] }
    }
}

fn main(x: Field, y: Field) {
    let first = Foo::default(x,y);
    assert(first == x);
}
