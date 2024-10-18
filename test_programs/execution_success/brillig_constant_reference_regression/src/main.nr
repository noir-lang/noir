unconstrained fn main(sorted_index: [u32; 2]) {
    let original = [55, 11];

    let mut sorted = original; // Stores the constant "original" into the sorted reference
    for i in 0..2 {
        let index = sorted_index[i];
        let value = original[index];
        sorted[i] = value; // On first iteration, we should not mutate the original constant array, RC should be > 1
    }

    assert_eq(sorted[1], 55);
}
