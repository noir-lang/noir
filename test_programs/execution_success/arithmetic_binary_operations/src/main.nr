// Tests a very simple program.
//
// The features being tested are:
// Binary addition, multiplication, division, constant modulo
// x = 3, y = 4, z = 5
fn main(x: Field, y: Field, z: Field) -> pub Field {
    //cast
    assert(y as u1 == 0);

    let a = x + x; // 3 + 3 = 6
    let b = a - y; // 6 - 4 = 2
    let c = b * z; // 2 * 5 = 10
    let d = c / a; // 10 / 6 (This uses field inversion, so we test it by multiplying by `a`)
    d * a
}
