#[foreign(sha256)]
fn sha256(_input : [u8]) -> [u8; 32] {}

#[foreign(blake2s)]
fn blake2s(_input : [u8]) -> [u8; 32] {}

#[foreign(pedersen)]
fn pedersen(_input : [Field]) -> [Field; 2] {}

#[foreign(hash_to_field)]
fn hash_to_field(_input : [Field]) -> Field {}

// mimc-p/p implementation
// constants are (publicly generated) random numbers, for instance using keccak as a ROM.
// You must use constants generated for the native field
// Rounds number should be ~ log(p)/log(exp)
// For 254 bit primes, exponent 7 and 91 rounds seems to be recommended
const ROUNDS: Field = 91;
fn mimc(x: Field, k: Field, constants: [Field;ROUNDS], exp : Field) -> Field {
    constrain ROUNDS == 91;     //TODO: to remove once global const is working fine
    //round 0
    let mut t = x + k;
    let mut h = crate::pow_32(t,exp);
    //next rounds
    for i in 1..91 {
        t = h + k + constants[i];
        h = crate::pow_32(t,exp);
    };
    h + k
}
