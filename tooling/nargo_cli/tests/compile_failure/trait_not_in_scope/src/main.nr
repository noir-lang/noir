use dep::std;

struct Foo {
    bar: Field,
    array: [Field; 2],
}

// Default trait does not exist
impl Default for Foo {
    fn default(x: Field, y: Field) -> Self {
        Self { bar: x, array: [x,y] }
    }
}

fn main(x: Field, y: Field) {
}
