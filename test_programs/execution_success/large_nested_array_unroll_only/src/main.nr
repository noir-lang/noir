// Test A: Large nested array with loop unrolling (no conditionals)
// This tests loop unrolling overhead without conditional merging.
fn main(input: [[u64; 3]; 1024]) {
    let mut sum: u64 = 0;
    for i in 0..1024 {
        sum += input[i][0];
    }
    assert(sum > 0);
}
