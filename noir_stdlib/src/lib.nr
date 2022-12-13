mod hash;
mod array;
mod merkle;
mod schnorr;
mod ecdsa_secp256k1;
mod scalar_mul;
mod sha256;
mod sha512;


#[builtin(to_bits)]
fn to_bits(_x : Field, _bit_size: u32) -> [u1; 256] {}

#[builtin(to_bytes)]
fn to_bytes(_x : Field, _byte_size: u32) -> [u8; 32] {}

// Returns base^exponent. 
// ^ means to the power of and not xor
// Caution: we assume the exponent fits into 32 bits
// using a bigger bit size impacts negatively the performance and should be done only if the exponent does not fit in 32 bits
fn pow_32(base: Field, exponent: Field) -> Field {
    let mut r = 1 as Field;
    let b = crate::to_bits(exponent, 32);

    for i in 1..33 {
        r = r*r;
        r = (b[32-i] as Field) * (r * base) + (1 - b[32-i] as Field) * r;
    };
    r
}

