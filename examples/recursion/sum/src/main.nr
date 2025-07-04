/// Adds two public 64-bit unsigned integers and returns their sum.
/// Demonstrates basic pub input/output in Noir.
fn main(a: pub u64, b: pub u64) -> pub u64 {
    a + b
}

#[test]
fn test_addition_correct_sum() {
    let result = main(7, 5);
    assert(result == 12);
}

// This test is expected to fail because 8 + 6 = 14 != 12
#[test(should_fail)]
fn test_addition_incorrect_sum() {
    let result = main(8, 6);
    assert(result == 12);
}
