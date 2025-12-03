unconstrained fn main() {
    // Test that for loops with inclusive ranges work correctly
    // even when the end value is the maximum for the type (u8::MAX = 255)
    let mut last_value = 0_u8;
    for x in 0_u8..=255_u8 {
        last_value = x;
    }
    // After the loop, last_value should be 255
    assert(last_value == 255);
}