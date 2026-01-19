// This program will fail when there is a specific difference between the two u64 elements
#[fuzz(only_fail_with = "broke_comparison_i64")]
fn comparison(a: i64, b: i64) {
    let c = b + 0xfffffffff0000;
    let d = c - 0xeeeeeeeeeeee;
    assert((a != d) | (b == 0) | (a == 0), "broke_comparison_i64");
}
