mod foo;
mod bar;

use crate::foo::foo;

fn main(x: Field, y: pub Field, z: Field) {
    let w = foo(x, y);
    assert(w == z);
}
