use std::hash::poseidon2;

global len: u32 = 2450 * 2 - 240; // for just under 2^17 gates
fn main(x: Field) {
    let ped_input = [x; len];
    let mut val = poseidon2::Poseidon2::hash(ped_input, len);
    let z = foo(x);
    assert(val == z);
}

#[fold]
fn foo(x: Field) -> Field {
    let ped_input = [x; len];
    let mut val = poseidon2::Poseidon2::hash(ped_input, len);
    val
}
