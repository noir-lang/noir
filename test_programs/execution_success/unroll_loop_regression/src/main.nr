global ROOTS: [Field; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

unconstrained fn main(fracs: [Field; 16]) -> pub [Field; 2] {
    let mut partial_sums: [Field; 2] = std::mem::zeroed();

    let mut partial_sum: Field = 0;
    for i in 0..8 {
        let summand = ROOTS[i] * fracs[i];
        partial_sum = partial_sum + summand;
    }
    partial_sums[0] = partial_sum;

    let NUM_PARTIAL_SUMS = 2;
    for i in 1..NUM_PARTIAL_SUMS {
        let mut partial_sum: Field = partial_sums[i - 1];
        for j in 0..8 {
            let k = i * 8 + j;
            let summand = ROOTS[k] * fracs[k];
            partial_sum = partial_sum + summand;
        }
        partial_sums[i] = partial_sum;
    }

    partial_sums
}
