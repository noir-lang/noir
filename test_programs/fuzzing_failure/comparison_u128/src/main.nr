// This program will fail when there is a specific difference between the two u128
#[fuzz(only_fail_with = "broke_comparison_u128")]
fn comparison(a: u128, b: u128) {
    let c = b * 3;
    let d = a * 6 + 1800000000000;
    assert((c != d) | (b == 0) | (a == 0), "broke_comparison_u128");
}
