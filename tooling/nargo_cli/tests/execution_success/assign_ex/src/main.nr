fn main(x: Field, y: Field) {
    let mut z = x + y;
    assert(z == 3);
    z = x * y;
    assert(z == 2);

    regression_3057();
}
// Ensure parsing parenthesized lvalues works
fn regression_3057() {
    let mut array = [[0, 1], [2, 3]];
    (array[0])[1] = 2;
    assert(array[0][1] == 2);
}
