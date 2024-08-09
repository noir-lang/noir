use std::embedded_curve_ops;

// Note: If main has any unsized types, then the verifier will never be able
// to figure out the circuit instance
fn main(
    message: [u8; 10],
    message_field: Field,
    pub_key_x: Field,
    pub_key_y: Field,
    signature: [u8; 64]
) {
    // Regression for issue #2421
    // We want to make sure that we can accurately verify a signature whose message is a slice vs. an array
    let message_field_bytes = message_field.to_be_bytes(10);
    let mut message2 = [0; 42];
    for i in 0..10 {
        assert(message[i] == message_field_bytes[i]);
        message2[i] = message[i];
    }

    // Is there ever a situation where someone would want 
    // to ensure that a signature was invalid?
    // Check that passing a slice as the message is valid
    let valid_signature = std::schnorr::verify_signature_slice(pub_key_x, pub_key_y, signature, message_field_bytes);
    assert(valid_signature);
    // Check that passing an array as the message is valid
    let valid_signature = std::schnorr::verify_signature(pub_key_x, pub_key_y, signature, message);
    assert(valid_signature);
    let pub_key = embedded_curve_ops::EmbeddedCurvePoint { x: pub_key_x, y: pub_key_y, is_infinite: false };
    let valid_signature = verify_signature_noir(pub_key, signature, message2);
    assert(valid_signature);
    assert_valid_signature(pub_key, signature, message2);
}

// TODO: to put in the stdlib once we have numeric generics
// Meanwhile, you have to use a message with 32 additional bytes:
// If you want to verify a signature on a message of 10 bytes, you need to pass a message of length 42,
// where the first 10 bytes are the one from the original message (the other bytes are not used)
pub fn verify_signature_noir<M>(public_key: embedded_curve_ops::EmbeddedCurvePoint, signature: [u8; 64], message: [u8; M]) -> bool {
    let N = message.len() - 32;

    //scalar lo/hi from bytes
    let sig_s = bytes_to_scalar(signature, 0);
    let sig_e = bytes_to_scalar(signature, 32);
    // pub_key is on Grumpkin curve
    let mut is_ok = (public_key.y * public_key.y == public_key.x * public_key.x * public_key.x - 17)
        & (!public_key.is_infinite);

    if ((sig_s.lo != 0) | (sig_s.hi != 0)) & ((sig_e.lo != 0) | (sig_e.hi != 0)) {
        let g1 = embedded_curve_ops::EmbeddedCurvePoint { x: 1, y: 17631683881184975370165255887551781615748388533673675138860, is_infinite: false };
        let r = embedded_curve_ops::multi_scalar_mul([g1, public_key], [sig_s, sig_e]);
        // compare the _hashes_ rather than field elements modulo r
        let pedersen_hash = std::hash::pedersen_hash([r.x, public_key.x, public_key.y]);
        let mut hash_input = [0; M];
        let pde = pedersen_hash.to_be_bytes(32);

        for i in 0..32 {
            hash_input[i] = pde[i];
        }
        for i in 0..N {
            hash_input[32+i] = message[i];
        }
        let result = std::hash::blake2s(hash_input);

        is_ok = !r.is_infinite;
        for i in 0..32 {
            if result[i] != signature[32 + i] {
                is_ok = false;
            }
        }
    }
    is_ok
}

pub fn bytes_to_scalar(bytes: [u8; 64], offset: u32) -> embedded_curve_ops::EmbeddedCurveScalar {
    let mut v = 1;
    let mut lo = 0 as Field;
    let mut hi = 0 as Field;
    for i in 0..16 {
        lo = lo + (bytes[offset+31 - i] as Field) * v;
        hi = hi + (bytes[offset+15 - i] as Field) * v;
        v = v * 256;
    }
    let sig_s = embedded_curve_ops::EmbeddedCurveScalar { lo, hi };
    sig_s
}

pub fn assert_valid_signature<M>(public_key: embedded_curve_ops::EmbeddedCurvePoint, signature: [u8; 64], message: [u8; M]) {
    let N = message.len() - 32;
    //scalar lo/hi from bytes
    let sig_s = bytes_to_scalar(signature, 0);
    let sig_e = bytes_to_scalar(signature, 32);

    // assert pub_key is on Grumpkin curve
    assert(public_key.y * public_key.y == public_key.x * public_key.x * public_key.x - 17);
    assert(public_key.is_infinite == false);
    // assert signature is not null
    assert((sig_s.lo != 0) | (sig_s.hi != 0));
    assert((sig_e.lo != 0) | (sig_e.hi != 0));

    let g1 = embedded_curve_ops::EmbeddedCurvePoint { x: 1, y: 17631683881184975370165255887551781615748388533673675138860, is_infinite: false };
    let r = embedded_curve_ops::multi_scalar_mul([g1, public_key], [sig_s, sig_e]);
    // compare the _hashes_ rather than field elements modulo r
    let pedersen_hash = std::hash::pedersen_hash([r.x, public_key.x, public_key.y]);
    let mut hash_input = [0; M];
    let pde = pedersen_hash.to_be_bytes(32);

    for i in 0..32 {
        hash_input[i] = pde[i];
    }
    for i in 0..N {
        hash_input[32+i] = message[i];
    }
    let result = std::hash::blake2s(hash_input);

    assert(!r.is_infinite);
    for i in 0..32 {
        assert(result[i] == signature[32 + i]);
    }
}
