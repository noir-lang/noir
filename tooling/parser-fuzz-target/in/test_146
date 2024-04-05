// This test checks that `var^var` is assigned the correct type.
// https://github.com/noir-lang/noir/issues/4080

#[test(should_fail_with = "attempt to add with overflow")]
fn main() {
    let var1: u8 = ((255 + 1) ^ (255 + 1)) - ((255 + 1) - (255 + 1));
    assert_eq(var1, 0);
}
