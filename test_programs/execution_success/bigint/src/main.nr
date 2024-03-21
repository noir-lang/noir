use dep::std::bigint;
use dep::std::{bigint::Secpk1Fq, println};

fn main(mut x: [u8; 5], y: [u8; 5]) {
    let a = bigint::Secpk1Fq::from_le_bytes([x[0], x[1], x[2], x[3], x[4]]);
    let b = bigint::Secpk1Fq::from_le_bytes([y[0], y[1], y[2], y[3], y[4]]);
    let a_bytes = a.to_le_bytes();
    let b_bytes = b.to_le_bytes();
    for i in 0..5 {
        assert(a_bytes[i] == x[i]);
        assert(b_bytes[i] == y[i]);
    }

    let d = a * b - b;
    let d1 = bigint::Secpk1Fq::from_le_bytes(597243850900842442924.to_le_bytes(10));
    assert(d1 == d);
    // big_int_example(x[0], x[1]);
}

// docs:start:big_int_example
fn big_int_example(x: u8, y: u8) {
    let a = Secpk1Fq::from_le_bytes([x, y, 0, 45, 2]);
    let b = Secpk1Fq::from_le_bytes([y, x, 9]);
    let c = (a + b) * b / a;
    let d = c.to_le_bytes();
    println(d[0]);
}
// docs:end:big_int_example
