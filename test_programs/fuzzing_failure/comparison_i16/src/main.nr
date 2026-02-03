// This program will fail when there is a specific difference between the two i16 elements
#[fuzz(only_fail_with = "broke_comparison_i16")]
fn comparison(a: i16, b: i16) {
    let c = b + 0xfff;
    let d = c - 0xeee;
    assert((a != d) | (b == 0) | (a == 0), "broke_comparison_i16");
}
