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
    let valid_signature = std::schnorr::verify_signature_noir(pub_key, signature, message);
    assert(valid_signature);
    std::schnorr::assert_valid_signature(pub_key, signature, message);
}

