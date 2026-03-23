// Test B: Large nested array with conditional in loop
// This tests loop unrolling + array merging from the if-then-else.
fn main(input: [[u64; 3]; 1024], condition: bool) {
    let mut arr = input;
    for i in 0..1024 {
        if condition {
            arr[i][0] = arr[i][0] + 1;
        }
    }
    assert(arr[0][0] > 0);
}
