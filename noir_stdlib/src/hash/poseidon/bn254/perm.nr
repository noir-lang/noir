// Instantiations of Poseidon permutation for the prime field of the same order as BN254
use crate::hash::poseidon::bn254::consts;
use crate::hash::poseidon::bn254::permute;
use crate::hash::poseidon::PoseidonConfig;

#[field(bn254)]
pub fn x5_2(mut state: [field; 2]) -> [field; 2] {
    state = permute(
        consts::x5_2_config(),
        state);

    state
}

#[field(bn254)]
pub fn x5_3(mut state: [field; 3]) -> [field; 3] {
    state = permute(
        consts::x5_3_config(),
        state);

    state
}

#[field(bn254)]
pub fn x5_4(mut state: [field; 4]) -> [field; 4] {
    state = permute(
        consts::x5_4_config(),
        state);

    state
}

#[field(bn254)]
pub fn x5_5(mut state: [field; 5]) -> [field; 5] {
    state = permute(
        consts::x5_5_config(),
        state);

    state
}

#[field(bn254)]
pub fn x5_6(mut state: [field; 6]) -> [field; 6] {
    state = permute(
        consts::x5_6_config(),
        state);

    state
}

#[field(bn254)]
pub fn x5_7(mut state: [field; 7]) -> [field; 7] {
    state = permute(
        consts::x5_7_config(),
        state);

    state
}

#[field(bn254)]
pub fn x5_8(mut state: [field; 8]) -> [field; 8] {
    state = permute(
        consts::x5_8_config(),
        state);

    state
}

#[field(bn254)]
pub fn x5_9(mut state: [field; 9]) -> [field; 9] {
    state = permute(
        consts::x5_9_config(),
        state);

    state
}

#[field(bn254)]
pub fn x5_10(mut state: [field; 10]) -> [field; 10] {
    state = permute(
        consts::x5_10_config(),
        state);

    state
}

#[field(bn254)]
pub fn x5_11(mut state: [field; 11]) -> [field; 11] {
    state = permute(
        consts::x5_11_config(),
        state);

    state
}

#[field(bn254)]
pub fn x5_12(mut state: [field; 12]) -> [field; 12] {
    state = permute(
        consts::x5_12_config(),
        state);

    state
}

#[field(bn254)]
pub fn x5_13(mut state: [field; 13]) -> [field; 13] {
    state = permute(
        consts::x5_13_config(),
        state);

    state
}

#[field(bn254)]
pub fn x5_14(mut state: [field; 14]) -> [field; 14] {
    state = permute(
        consts::x5_14_config(),
        state);

    state
}

#[field(bn254)]
pub fn x5_15(mut state: [field; 15]) -> [field; 15] {
    state = permute(
        consts::x5_15_config(),
        state);

    state
}

#[field(bn254)]
pub fn x5_16(mut state: [field; 16]) -> [field; 16] {
    state = permute(
        consts::x5_16_config(),
        state);

    state
}

#[field(bn254)]
pub fn x5_17(mut state: [field; 17]) -> [field; 17] {
    state = permute(
        consts::x5_17_config(),
        state);

    state
}
