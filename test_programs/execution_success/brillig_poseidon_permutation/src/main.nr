use std::hash::poseidon2_permutation;

unconstrained fn main(input1: [Field; 16], input2: [Field; 4], flag: bool) -> pub Field {
    if flag {
        // If enabled, this would fail, because we expect the input length to be 4,
        // however in Brillig this call will never be made, and the circuit passes.
        // In ACIR it will fail even if disabled (see execution_failure/poseidon_permutation).
        poseidon2_permutation(input1, 16)[0]
    } else {
        poseidon2_permutation(input2, 4)[0]
    }
}
