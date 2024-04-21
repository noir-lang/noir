use dep::std::bigint;
use dep::std::{bigint::Secpk1Fq, println};

fn main(mut x: [u8; 5], y: [u8; 5]) {
    let a = bigint::Secpk1Fq::from_le_bytes(&[x[0], x[1], x[2], x[3], x[4]]);
    let b = bigint::Secpk1Fq::from_le_bytes(&[y[0], y[1], y[2], y[3], y[4]]);
    let mut a_be_bytes = [0; 32];
    let mut b_be_bytes = [0; 32];
    for i in 0..5 {
        a_be_bytes[31-i] = x[i];
        b_be_bytes[31-i] = y[i];
    }
    let a_field = dep::std::field::bytes32_to_field(a_be_bytes);
    let b_field = dep::std::field::bytes32_to_field(b_be_bytes);

    // Regression for #4682
    let c = if x[0] != 0 {
        test_unconstrained1(a, b)
    } else {
        test_unconstrained2(a, b)
    };
    assert(c.array[0] == dep::std::wrapping_mul(x[0], y[0]));

    let a_bytes = a.to_le_bytes();
    let b_bytes = b.to_le_bytes();
    for i in 0..5 {
        assert(a_bytes[i] == x[i]);
        assert(b_bytes[i] == y[i]);
    }
    //Regression for issue #4578
    let d = a * b;
    assert(d / b == a);

    let d = d - b;
    let mut result = [0; 32];
    let result_slice = (a_field * b_field - b_field).to_le_bytes(32);
    for i in 0..32 {
        result[i] = result_slice[i];
    }
    let d1 = bigint::Secpk1Fq::from_le_bytes_32(result);
    assert(d1 == d);
    big_int_example(x[0], x[1]);
}

fn test_unconstrained1(a: Secpk1Fq, b: Secpk1Fq) -> Secpk1Fq {
    let c = a * b;
    c
}
unconstrained fn test_unconstrained2(a: Secpk1Fq, b: Secpk1Fq) -> Secpk1Fq {
    let c = a + b;
    test_unconstrained1(a, c)
}

// docs:start:big_int_example
fn big_int_example(x: u8, y: u8) {
    let a = Secpk1Fq::from_le_bytes(&[x, y, 0, 45, 2]);
    let b = Secpk1Fq::from_le_bytes(&[y, x, 9]);
    let c = (a + b) * b / a;
    let d = c.to_le_bytes();
    println(d[0]);
}
// docs:end:big_int_example
