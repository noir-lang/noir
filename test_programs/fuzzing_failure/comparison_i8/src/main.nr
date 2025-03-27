// This program will fail when there is a specific difference between the two i8 elements
#[fuzz(only_fail_with = "broke_comparison_i8")]
fn comparison(a: i8, b: i8) {
    let c = b + 0xf;
    let d = c - 0xe;
    assert((a != d) | (b == 0) | (a == 0), "broke_comparison_i8");
}
