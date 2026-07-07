// This program expects the non-deadbeef failure message, so the deadbeef case is treated as an actual failure
#[fuzz(only_fail_with = "input is not 0xdeadbeef")]
fn fuzz(input: u64) {
    assert(input == 0xdeadbeef, "input is not 0xdeadbeef");
    assert(input != 0xdeadbeef, "input is 0xdeadbeef");
}
