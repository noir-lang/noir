use dep::std;

struct Foo {
    bar: Field,
    array: [Field; 2],
}

struct Default {
    x: Field,
    z: Field, 
}

// Default is struct not a trait
impl Default for Foo {
    fn default(x: Field, y: Field) -> Self {
        Self { bar: x, array: [x,y] }
    }
}

fn main(x: Field, y: Field) {
    let first = Foo::default(x,y);
    assert(first.bar == x);
}
