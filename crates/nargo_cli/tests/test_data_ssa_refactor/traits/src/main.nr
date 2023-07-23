use dep::std;

trait Default {
    fn default(x: Field, y: Field) -> Self;
}

struct Foo {
    bar: Field,
    array: [Field; 2],
}


impl Default for Foo {
    fn default(x: Field,y: Field) -> Self {
        Self { bar: 0, array: [x,y] }
    }
}

fn main(x: Field, y: Field) {
    let first = Foo::default(x,y);
    assert(first.bar == x);
}
