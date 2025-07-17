#[test(should_fail_with = "42 is not allowed")]
fn finds_magic_value(x: u32) {
    let x = x as u64;
    if x == 21 {
        assert(2 * x != 42, "42 is not allowed");
    } else {
        assert(2 * x == 42, "42 is not allowed");
    }
}
