// Instantiations of Poseidon constants, permutations and sponge for prime field of the same order as BN254
mod perm;
mod consts;

use crate::hash::poseidon::{PoseidonConfig, absorb};

// Variable-length Poseidon-128 sponge as suggested in second bullet point of §3 of https://eprint.iacr.org/2019/458.pdf
#[field(bn254)]
pub fn sponge<N>(msg: [Field; N]) -> Field {
    absorb(consts::x5_5_config(), [0; 5], 4, 1, msg)[1]
}

// Various instances of the Poseidon hash function
// Consistent with Circom's implementation
pub fn hash_1(input: [Field; 1]) -> Field {
    let mut state = [0; 2];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_2(state)[0]
}

pub fn hash_2(input: [Field; 2]) -> Field {
    let mut state = [0; 3];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_3(state)[0]
}

pub fn hash_3(input: [Field; 3]) -> Field {
    let mut state = [0; 4];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_4(state)[0]
}

pub fn hash_4(input: [Field; 4]) -> Field {
    let mut state = [0; 5];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_5(state)[0]
}

pub fn hash_5(input: [Field; 5]) -> Field {
    let mut state = [0; 6];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_6(state)[0]
}

pub fn hash_6(input: [Field; 6]) -> Field {
    let mut state = [0; 7];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_7(state)[0]
}

pub fn hash_7(input: [Field; 7]) -> Field {
    let mut state = [0; 8];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_8(state)[0]
}

pub fn hash_8(input: [Field; 8]) -> Field {
    let mut state = [0; 9];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_9(state)[0]
}

pub fn hash_9(input: [Field; 9]) -> Field {
    let mut state = [0; 10];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_10(state)[0]
}

pub fn hash_10(input: [Field; 10]) -> Field {
    let mut state = [0; 11];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_11(state)[0]
}

pub fn hash_11(input: [Field; 11]) -> Field {
    let mut state = [0; 12];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_12(state)[0]
}

pub fn hash_12(input: [Field; 12]) -> Field {
    let mut state = [0; 13];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_13(state)[0]
}

pub fn hash_13(input: [Field; 13]) -> Field {
    let mut state = [0; 14];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_14(state)[0]
}

pub fn hash_14(input: [Field; 14]) -> Field {
    let mut state = [0; 15];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_15(state)[0]
}

pub fn hash_15(input: [Field; 15]) -> Field {
    let mut state = [0; 16];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_16(state)[0]
}

pub fn hash_16(input: [Field; 16]) -> Field {
    let mut state = [0; 17];
    for i in 0..input.len() {
        state[i+1] = input[i];
    }

    perm::x5_17(state)[0]
}
