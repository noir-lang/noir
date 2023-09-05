use dep::std;

trait Default {
    fn default(x: Field, y: Field) -> Self;
}

struct Foo {
    bar: Field,
    array: [Field; 2],
}

impl Default for Foo {
    fn default(x: Field) -> Self {
        Self { bar: x, array: [x, x] }
    }
}

fn main(x: Field, y: Field) {
}
