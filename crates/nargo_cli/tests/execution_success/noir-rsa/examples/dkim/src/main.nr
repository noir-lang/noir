mod biguint;
use crate::biguint::BigUint56;

fn main(
    dkim_sig: BigUint56,
    modulus: BigUint56
) -> pub [u8;259]{
    let padded_sha256_hash = dkim_sig.mulmod(dkim_sig, modulus);
    padded_sha256_hash.to_bytes()
}
