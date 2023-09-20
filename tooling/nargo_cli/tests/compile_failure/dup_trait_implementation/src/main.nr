use dep::std;

trait Default {
    fn default(x: Field, y: Field) -> Field;
}

struct Foo {
    bar: Field,
    array: [Field; 2],
}

impl Default for Foo {
    // Duplicate trait methods should not compile
    fn default(x: Field, y: Field) -> Field {
        y + 2 * x
    }
    // Duplicate trait methods should not compile
    fn default(x: Field, y: Field) -> Field {
        y + 2 * x
    }
}



fn main(x: Field, y: Field) {
}
