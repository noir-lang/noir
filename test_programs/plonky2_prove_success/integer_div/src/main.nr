// Testing just integer division: 7 / 3 = 2
//
// At the moment, this test causes a crash inside plonky2::prove, because the
// result of 7 / 3 is not 2 there. Perhaps PLONKY2 should return an error from
// prove, rather than crash.
fn main(x: u32, y: u32, z: u32) {
    assert(x / y == z);
}
