mod hash;
mod array;
mod merkle;
mod schnorr;
mod ecdsa_secp256k1;

#[builtin(set_pub)]
fn set_as_public(_input : Field) {}