// Regression test for noir-lang/noir-claude#1729: the shift amounts only become constants after
// loop unrolling, and the shifts by the full bit size sit behind `c`, which is false here.
fn main(y: i8, z: i32, c: bool) -> pub [i8; 3] {
    let mut a: i8 = 0;
    for i in 7..9 {
        let s = i as i8;
        if c {
            a = y >> s;
        }
    }
    let mut b: i32 = 0;
    for i in 31..33 {
        let s = i as i32;
        if c {
            b = z >> s;
        }
    }
    assert_eq(z >> 31, -1);
    [a, y >> 7, b as i8]
}
