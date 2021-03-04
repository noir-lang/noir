#[foreign(schnorr_verify)]
fn verify_signature(_signature :[64]u8, _message : []u8, _public_key_x : Field, _public_key_y : Field) -> Field {}