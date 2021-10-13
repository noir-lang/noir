
//extern crate barretenberg_wrapper;
use barretenberg_wrapper::blake2s;
use barretenberg_wrapper::pedersen;

pub fn main() {
    let input = vec![0; 64];
    let mut r = [0_u8; 32];
    blake2s::hash_to_field(&input);
    
    let f_zero = [0_u8; 32];
    let mut f_one = [0_u8; 32]; f_one[31] = 1;
    let got = pedersen::compress_native(&f_zero, &f_one);
    println!("pederson: {}",hex::encode(got));
}