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
        Self { bar: x, array: [x,y] }
    }
}

// Duplicate trait declarations should not compile
trait Default {
    fn default(x: Field) -> Self;
}

fn main(x: Field, y: Field) {
    let first = Foo::default(x,y);
    assert(first.bar == x);
}
