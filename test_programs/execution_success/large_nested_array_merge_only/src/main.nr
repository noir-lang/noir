// Test C: Single conditional merge of large array (no loop)
// This isolates just the merge cost without unrolling.
fn main(input: [[u64; 3]; 1024], condition: bool) {
    let arr = if condition { input } else { input };
    assert(arr[0][0] >= 0);
}
