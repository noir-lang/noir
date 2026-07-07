// This program will fail when there is a specific difference between the two FieldElements
#[fuzz(only_fail_with = "assertion failed")]
fn comparison(a: Field, b: Field) {
    let c = b * 3;
    // let d = a * 6 + 0xdeadbeefc0ffee * 6; // Very slow on CI.
    let d = a * 6 + 0xf;
    assert((c != d) | (b == 0) | (a == 0), "assertion failed");
}
