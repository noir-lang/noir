use dep::std;

trait Default {
}

struct Foo {
    bar: Field,
    array: [Field; 2],
}

// wrong trait name method should not compile
impl Default for Foo {
    fn doesnt_exist(x: Field, y: Field) -> Self {
        Self { bar: x, array: [x,y] }
    }
}

fn main(x: Field, y: Field) {
}
