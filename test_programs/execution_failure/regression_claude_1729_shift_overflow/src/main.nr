// Regression test for noir-lang/noir-claude#1729: a signed shift by a loop-unrolled constant equal
// to the bit size must fail the bit-shift overflow check.
fn main(y: i8, c: bool) -> pub i8 {
    let mut a: i8 = 0;
    for i in 7..9 {
        let s = i as i8;
        if c {
            a = y >> s;
        }
    }
    a
}
