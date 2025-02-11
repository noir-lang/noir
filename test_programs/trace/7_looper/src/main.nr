
fn main(x: Field, y: pub Field, z: Field) {
    let w = looper(x, y);
    assert(w == z);
}

fn looper(x: Field, y: Field) -> Field {
    let mut result = x;
    for i in 0..10 {
        println(i + 1);
        if i % 2 == 0 {
            result = result + y;
        } else {
            result = result - y;
        }
    }
    result
}
