// Instantiations of Poseidon permutation for the prime field of the same order as BN254
use crate::hash::poseidon::bn254::consts;
use crate::hash::poseidon::permute;

fn x5_2(state: [Field; 2]) -> [Field; 2]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[0],
                       consts::alpha,
                       consts::ark2(),
                       consts::mds2(),
                       state);

    for i in 0..2
    {
        out[i] = perm[i];
    }

    out
}

fn x5_3(state: [Field; 3]) -> [Field; 3]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[1],
                       consts::alpha,
                       consts::ark3(),
                       consts::mds3(),
                       state);

    for i in 0..3
    {
        out[i] = perm[i];
    }

    out
}

fn x5_4(state: [Field; 4]) -> [Field; 4]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[2],
                       consts::alpha,
                       consts::ark4(),
                       consts::mds4(),
                       state);

    for i in 0..4
    {
        out[i] = perm[i];
    }

    out
}

fn x5_5(state: [Field; 5]) -> [Field; 5]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[3],
                       consts::alpha,
                       consts::ark5(),
                       consts::mds5(),
                       state);

    for i in 0..5
    {
        out[i] = perm[i];
    }

    out
}

fn x5_6(state: [Field; 6]) -> [Field; 6]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[4],
                       consts::alpha,
                       consts::ark6(),
                       consts::mds6(),
                       state);

    for i in 0..6
    {
        out[i] = perm[i];
    }

    out
}

fn x5_7(state: [Field; 7]) -> [Field; 7]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[5],
                       consts::alpha,
                       consts::ark7(),
                       consts::mds7(),
                       state);

    for i in 0..7
    {
        out[i] = perm[i];
    }

    out
}

fn x5_8(state: [Field; 8]) -> [Field; 8]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[6],
                       consts::alpha,
                       consts::ark8(),
                       consts::mds8(),
                       state);

    for i in 0..8
    {
        out[i] = perm[i];
    }

    out
}

fn x5_9(state: [Field; 9]) -> [Field; 9]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[7],
                       consts::alpha,
                       consts::ark9(),
                       consts::mds9(),
                       state);

    for i in 0..9
    {
        out[i] = perm[i];
    }

    out
}

fn x5_10(state: [Field; 10]) -> [Field; 10]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[8],
                       consts::alpha,
                       consts::ark10(),
                       consts::mds10(),
                       state);

    for i in 0..10
    {
        out[i] = perm[i];
    }

    out
}

fn x5_11(state: [Field; 11]) -> [Field; 11]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[9],
                       consts::alpha,
                       consts::ark11(),
                       consts::mds11(),
                       state);

    for i in 0..11
    {
        out[i] = perm[i];
    }

    out
}

fn x5_12(state: [Field; 12]) -> [Field; 12]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[10],
                       consts::alpha,
                       consts::ark12(),
                       consts::mds12(),
                       state);

    for i in 0..12
    {
        out[i] = perm[i];
    }

    out
}

fn x5_13(state: [Field; 13]) -> [Field; 13]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[11],
                       consts::alpha,
                       consts::ark13(),
                       consts::mds13(),
                       state);

    for i in 0..13
    {
        out[i] = perm[i];
    }

    out
}

fn x5_14(state: [Field; 14]) -> [Field; 14]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[12],
                       consts::alpha,
                       consts::ark14(),
                       consts::mds14(),
                       state);

    for i in 0..14
    {
        out[i] = perm[i];
    }

    out
}

fn x5_15(state: [Field; 15]) -> [Field; 15]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[13],
                       consts::alpha,
                       consts::ark15(),
                       consts::mds15(),
                       state);

    for i in 0..15
    {
        out[i] = perm[i];
    }

    out
}

fn x5_16(state: [Field; 16]) -> [Field; 16]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[14],
                       consts::alpha,
                       consts::ark16(),
                       consts::mds16(),
                       state);

    for i in 0..16
    {
        out[i] = perm[i];
    }

    out
}

fn x5_17(state: [Field; 17]) -> [Field; 17]
{
    let mut out = state;
    let perm = permute(consts::rf,
                       consts::rp()[15],
                       consts::alpha,
                       consts::ark17(),
                       consts::mds17(),
                       state);

    for i in 0..17
    {
        out[i] = perm[i];
    }

    out
}
