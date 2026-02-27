use std::hash::poseidon2_permutation;

fn main(input1: [Field; 16], input2: [Field; 4], flag: bool) -> pub Field {
    if flag {
        // Even if disabled, this call will cause the circuit to fail,
        // because this opcode does not take an ACIR predicate.
        // In Brillig this wouldn't cause a failure (see execution_success/brillig_poseidon_permutation),
        // so we make sure we don't run this test with --force-brillig.
        poseidon2_permutation(input1, 16)[0]
    } else {
        poseidon2_permutation(input2, 4)[0]
    }
}
