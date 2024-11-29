global foo = -1;

fn main(x: i32) {
    let y = x + foo;
    assert(y == 0);
}
