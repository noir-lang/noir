// This program will fail when there is a specific difference between the two u32 elements
#[fuzz(only_fail_with = "broke_comparison_u32")]
fn comparison(a: u32, b: u32) {
    let c = b + 0xfffffff;
    let d = c - 0xeeeeeee;
    assert((a != d) | (b == 0) | (a == 0), "broke_comparison_u32");
}
