use dep::std;

trait Default {
    fn default(x: Field, y: Field) -> Self;
}

struct Foo {
    bar: Field,
    array: [Field; 2],
}

impl Default for Foo {
    fn default(x: Field, y: Foo) -> Self {
        Self { bar: x, array: [x, y.bar] }
    }
}

fn main(x: Field, y: Field) {
}
