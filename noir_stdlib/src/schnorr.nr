#[foreign(schnorr_verify)]
fn verify_signature(_public_key_x: Field, _public_key_y: Field, _signature: [u8; 64], _message: [u8]) -> Field {}
