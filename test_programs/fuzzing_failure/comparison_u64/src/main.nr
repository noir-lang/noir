// This program will fail when there is a specific difference between the two u64 elements
#[fuzz(only_fail_with = "broke_comparison_u64")]
fn comparison(a: u64, b: u64) {
    let c = b + 0xffffffffffff0000;
    let d = c - 0xeeeeeeeeeeeeee;
    assert((a != d) | (b == 0) | (a == 0), "broke_comparison_u64");
}
