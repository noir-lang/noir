
fn main(x : Field, y : Field) {
    priv z = x + y;
    constrain z == 3;
    z = x * y;
    constrain z == 2;
}