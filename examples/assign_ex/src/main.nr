
fn main(x : Field, y : Field) {
    let mut z = x + y;
    constrain z == 3;
    z = x * y;
    constrain z == 2;
}
