use dep::std;

trait Default {
    fn default(x: Field, y: Field) -> Self;
}

struct Foo {
    bar: Field,
    array: [Field; 2],
}

// wrong trait name method should not compile
impl Default for Foo {
    fn default_wrong_name(x: Field, y: Field) -> Self {
        Self { bar: x, array: [x,y] }
    }
}

fn main(x: Field, y: Field) {
    let first = Foo::default_wrong_name(x,y);
    assert(first.bar == x);
}
