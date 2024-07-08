use crate::collections::vec::Vec;
use crate::embedded_curve_ops::{EmbeddedCurvePoint, EmbeddedCurveScalar};

#[foreign(schnorr_verify)]
// docs:start:schnorr_verify
pub fn verify_signature<let N: u32>(
    public_key_x: Field,
    public_key_y: Field,
    signature: [u8; 64],
    message: [u8; N]
) -> bool
// docs:end:schnorr_verify
{}

#[foreign(schnorr_verify)]
// docs:start:schnorr_verify_slice
pub fn verify_signature_slice(
    public_key_x: Field,
    public_key_y: Field,
    signature: [u8; 64],
    message: [u8]
) -> bool
// docs:end:schnorr_verify_slice
{}

pub fn verify_signature_noir<let N: u32>(public_key: EmbeddedCurvePoint, signature: [u8; 64], message: [u8; N]) -> bool {
    //scalar lo/hi from bytes
    let sig_s = EmbeddedCurveScalar::from_bytes(signature, 0);
    let sig_e = EmbeddedCurveScalar::from_bytes(signature, 32);
    // pub_key is on Grumpkin curve
    let mut is_ok = (public_key.y * public_key.y == public_key.x * public_key.x * public_key.x - 17)
        & (!public_key.is_infinite);

    if ((sig_s.lo != 0) | (sig_s.hi != 0)) & ((sig_e.lo != 0) | (sig_e.hi != 0)) {
        let g1 = EmbeddedCurvePoint { x: 1, y: 17631683881184975370165255887551781615748388533673675138860, is_infinite: false };
        let r = crate::embedded_curve_ops::multi_scalar_mul([g1, public_key], [sig_s, sig_e]);
        // compare the _hashes_ rather than field elements modulo r
        let pedersen_hash = crate::hash::pedersen_hash([r[0], public_key.x, public_key.y]);
        let pde = pedersen_hash.to_be_bytes(32);

        let mut hash_input = Vec::new();
        for i in 0..32 {
            hash_input.push(pde[i]);
        }
        for i in 0..N {
            hash_input.push(message[i]);
        }

        let result = crate::hash::blake2s_slice(hash_input.slice);
        is_ok = (r[2] == 0);
        for i in 0..32 {
            if result[i] != signature[32 + i] {
                is_ok = false;
            }
        }
    }
    is_ok
}

pub fn assert_valid_signature<let N: u32>(public_key: EmbeddedCurvePoint, signature: [u8; 64], message: [u8; N]) {
    //scalar lo/hi from bytes
    let sig_s = EmbeddedCurveScalar::from_bytes(signature, 0);
    let sig_e = EmbeddedCurveScalar::from_bytes(signature, 32);

    // assert pub_key is on Grumpkin curve
    assert(public_key.y * public_key.y == public_key.x * public_key.x * public_key.x - 17);
    assert(public_key.is_infinite == false);
    // assert signature is not null
    assert((sig_s.lo != 0) | (sig_s.hi != 0));
    assert((sig_e.lo != 0) | (sig_e.hi != 0));

    let g1 = EmbeddedCurvePoint { x: 1, y: 17631683881184975370165255887551781615748388533673675138860, is_infinite: false };
    let r = crate::embedded_curve_ops::multi_scalar_mul([g1, public_key], [sig_s, sig_e]);
    // compare the _hashes_ rather than field elements modulo r
    let pedersen_hash = crate::hash::pedersen_hash([r[0], public_key.x, public_key.y]);
    let pde = pedersen_hash.to_be_bytes(32);

    let mut hash_input = Vec::new();
    for i in 0..32 {
        hash_input.push(pde[i]);
    }
    for i in 0..N {
        hash_input.push(message[i]);
    }

    let result = crate::hash::blake2s_slice(hash_input.slice);

    assert(r[2] == 0);
    for i in 0..32 {
        assert(result[i] == signature[32 + i]);
    }
}
