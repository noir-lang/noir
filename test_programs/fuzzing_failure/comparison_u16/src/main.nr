// This program will fail when there is a specific difference between the two u16 elements
#[fuzz(only_fail_with = "broke_comparison_u16")]
fn comparison(a: u16, b: u16) {
    let c = b + 0xfff;
    let d = c - 0xeee;
    assert((a != d) | (b == 0) | (a == 0), "broke_comparison_u16");
}
