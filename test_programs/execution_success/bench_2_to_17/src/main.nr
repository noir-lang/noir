use std::hash::poseidon2;

global len: u32 = 2450 * 2;
fn main(x: Field) {
    let ped_input = [x; len];
    let mut val = poseidon2::Poseidon2::hash(ped_input, len);
    assert(val != 0);
}
