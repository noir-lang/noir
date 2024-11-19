//! Tests to show that the comptime interpreter implement blackbox functions.
use std::bigint;

#[test]
fn test_bigint() {
    let result: [u8] = comptime {
        let a = bigint::Secpk1Fq::from_le_bytes(&[0, 1, 2, 3, 4]);
        let b = bigint::Secpk1Fq::from_le_bytes(&[5, 6, 7, 8, 9]);
        let c = 1 + 2 + 3;
        println(f"result0 = {c}");
        let c = a + a;
        println(f"result1 = {c}");
        // TODO: It cannot add c to itself
        let c = a + c;
        println(f"result2 = {c}");
        // TODO: It cannot add 3 bigints in a single expression.
        let c = a + a + a;
        println(f"result3 = {c}");
        let c = (a + b) * b / a - a;
        c.to_le_bytes()
    };

    println(f"result = {result}");

    // let expected = &[0, 1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    // assert_eq(result, expected);
}
