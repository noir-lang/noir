
use bignum::{BigNum, BN254_Fq};


// https://github.com/noir-lang/noir/issues/8210
fn main() {
    BN254_Fq::zero().modulus_bits();
}

