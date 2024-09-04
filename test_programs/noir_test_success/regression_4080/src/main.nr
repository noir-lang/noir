// This test checks that `var^var` is assigned the correct type.
// https://github.com/noir-lang/noir/issues/4080

// NOTE: expected error is the following, but the 'failed_assertion' is None in check_expected_failure_message
#[test(should_fail_with = "Cannot solve opcode: FunctionInput value has too many bits")]
fn main() {
    let var1: u8 = ((255 + 1) ^ (255 + 1)) - ((255 + 1) - (255 + 1));
    assert_eq(var1, 0);
}
